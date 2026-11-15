use std::{
    collections::{HashMap, HashSet},
    ops::Range,
    path::Path,
    sync::Arc,
    time::{Duration, Instant},
};

use crate::{
    artwork_cache::{ArtworkCache, ReadFailure, load_indexed},
    catalogue::{IndexedTrack, Library},
    dropbox::ArtworkAccess,
    track::Track,
};
use iced::{
    Task,
    widget::{image::Handle, scrollable::Viewport},
};

use super::super::{CataloguePaths, LibraryPage, Message, Snapshot};
use super::{Covers, ROW_LIMIT, row_handle};

const CONCURRENT_READS: usize = 2;
const DEFAULT_ROWS: Range<usize> = 0..16;
const OVERSCAN_ROWS: usize = 4;
const RETRY_DELAY: Duration = Duration::from_secs(30);

#[derive(Default)]
pub(in crate::desktop) struct Loader {
    generation: u64,
    visible: Option<(LibraryPage, Range<usize>)>,
    queue_visible: Option<Range<usize>>,
    pending: HashSet<(u64, String)>,
    attempted: HashMap<String, Option<Instant>>,
    remote: Arc<ArtworkAccess>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::desktop) enum Target {
    Library,
    Queue,
}

#[derive(Debug, Clone)]
pub struct Complete {
    generation: u64,
    pub(in crate::desktop) id: String,
    pub(in crate::desktop) target: Target,
    cover: Result<Option<Handle>, ReadFailure>,
}

struct Candidate<'a> {
    id: &'a str,
    library: &'a str,
    path: &'a Path,
    remote: bool,
}

impl<'a> Candidate<'a> {
    fn indexed(track: &'a IndexedTrack, libraries: &[Library]) -> Self {
        Self {
            id: &track.id,
            library: &track.library_id,
            path: &track.path,
            remote: libraries
                .iter()
                .find(|source| source.id == track.library_id)
                .is_some_and(|source| source.kind == "dropbox"),
        }
    }

    fn queued(track: &'a Track) -> Option<Self> {
        let (path, remote) = track
            .remote()
            .map_or((track.path(), false), |(_, path)| (Path::new(path), true));
        Some(Self {
            id: track.indexed_id()?,
            library: track.library()?,
            path,
            remote,
        })
    }
}

pub(in crate::desktop) fn viewport_rows(
    viewport: Viewport,
    row_height: f32,
    inset: f32,
    extra_content_height: f32,
) -> Range<usize> {
    let top = (viewport.absolute_offset().y - inset).max(0.0);
    let bottom = top + viewport.bounds().height;
    // An expanded editor or picker can sit before or after the viewport.
    let first_possible_row = (top - extra_content_height).max(0.0);
    (first_possible_row / row_height).floor() as usize..(bottom / row_height).ceil() as usize
}

impl Loader {
    pub(in crate::desktop) fn catalogue_changed(&mut self) {
        self.generation = self.generation.wrapping_add(1);
        self.attempted.clear();
    }

    pub(in crate::desktop) fn visible(&mut self, page: LibraryPage, rows: Range<usize>) {
        if self
            .visible
            .as_ref()
            .is_some_and(|(previous, _)| *previous != page)
        {
            self.attempted.retain(|_, retry| retry.is_none());
        }
        self.visible = Some((page, rows));
    }

    pub(in crate::desktop) fn complete(
        &mut self,
        result: Complete,
        covers: Option<&mut Covers>,
    ) -> Option<(String, Handle)> {
        self.pending.remove(&(result.generation, result.id.clone()));
        if result.generation != self.generation {
            return None;
        }
        match result.cover {
            Ok(Some(cover)) => {
                self.attempted.remove(&result.id);
                if let Some(covers) = covers
                    && (covers.len() < ROW_LIMIT || covers.contains_key(&result.id))
                {
                    covers.insert(result.id.clone(), cover.clone());
                }
                Some((result.id, cover))
            }
            Ok(None) => None,
            Err(_) => {
                self.attempted
                    .insert(result.id, Some(Instant::now() + RETRY_DELAY));
                None
            }
        }
    }

    pub(in crate::desktop) fn queue_visible(&mut self, rows: Option<Range<usize>>) {
        self.queue_visible = rows;
    }

    pub(in crate::desktop) fn request(
        &mut self,
        paths: &CataloguePaths,
        snapshot: &mut Snapshot,
        page: LibraryPage,
    ) -> Task<Message> {
        let available = CONCURRENT_READS.saturating_sub(self.pending.len());
        if available == 0 || page == LibraryPage::Settings {
            return Task::none();
        }
        let visible = self
            .visible
            .as_ref()
            .filter(|(visible_page, _)| *visible_page == page)
            .map_or(DEFAULT_ROWS, |(_, rows)| rows.clone());
        let last = if page == LibraryPage::Songs {
            snapshot.tracks.len()
        } else {
            snapshot.playlist_entries.len()
        };
        let candidates = visible_indices(visible, last)
            .filter_map(|index| {
                if page == LibraryPage::Songs {
                    snapshot.tracks.get(index)
                } else {
                    snapshot.playlist_entries.get(index)?.track.as_ref()
                }
            })
            .map(|track| Candidate::indexed(track, &snapshot.libraries))
            .collect();
        self.schedule(paths, &mut snapshot.covers, candidates, Target::Library)
    }

    pub(in crate::desktop) fn request_queue(
        &mut self,
        paths: &CataloguePaths,
        tracks: &[Track],
        covers: &mut Covers,
    ) -> Task<Message> {
        let visible = self.queue_visible.clone().unwrap_or(DEFAULT_ROWS);
        let candidates = visible_indices(visible, tracks.len())
            .filter_map(|index| Candidate::queued(&tracks[index]))
            .collect();
        self.schedule(paths, covers, candidates, Target::Queue)
    }

    fn schedule(
        &mut self,
        paths: &CataloguePaths,
        covers: &mut Covers,
        candidates: Vec<Candidate<'_>>,
        target: Target,
    ) -> Task<Message> {
        let available = CONCURRENT_READS.saturating_sub(self.pending.len());
        if available == 0 {
            return Task::none();
        }
        if self.attempted.len().max(covers.len()) + candidates.len() > ROW_LIMIT {
            let visible_ids: HashSet<_> = candidates.iter().map(|track| track.id).collect();
            self.attempted
                .retain(|id, _| visible_ids.contains(id.as_str()));
            covers.retain(|id, _| visible_ids.contains(id.as_str()));
        }
        let now = Instant::now();
        let mut requests = Vec::new();
        for track in candidates {
            let attempted = self
                .attempted
                .get(track.id)
                .is_some_and(|retry| retry.is_none_or(|time| time > now));
            if covers.contains_key(track.id)
                || attempted
                || self
                    .pending
                    .contains(&(self.generation, track.id.to_string()))
            {
                continue;
            }
            self.attempted.insert(track.id.to_string(), None);
            self.pending.insert((self.generation, track.id.to_string()));
            requests.push(self.read(paths.clone(), track, target));
            if requests.len() == available {
                break;
            }
        }
        Task::batch(requests)
    }

    fn read(&self, paths: CataloguePaths, track: Candidate<'_>, target: Target) -> Task<Message> {
        let reader = Arc::clone(&self.remote);
        let generation = self.generation;
        let id = track.id.to_string();
        let library = track.library.to_string();
        let path = track.path.to_path_buf();
        let remote = track.remote;
        Task::perform(
            async move {
                let track_id = id.clone();
                let cover = tokio::task::spawn_blocking(move || {
                    load(
                        &paths,
                        &reader,
                        Candidate {
                            id: &track_id,
                            library: &library,
                            path: &path,
                            remote,
                        },
                    )
                })
                .await
                .map_err(|_| ReadFailure::Worker)
                .and_then(|result| result);
                Complete {
                    generation,
                    id,
                    target,
                    cover,
                }
            },
            Message::ArtworkReady,
        )
    }
}

fn visible_indices(visible: Range<usize>, last: usize) -> impl Iterator<Item = usize> {
    let visible = if visible.start >= last || last <= visible.len() {
        0..visible.len().max(DEFAULT_ROWS.end)
    } else {
        visible
    };
    let first = visible.start.min(last);
    let end = visible.end.min(last);
    (first..end)
        .chain(first.saturating_sub(OVERSCAN_ROWS)..first)
        .chain(end..end.saturating_add(OVERSCAN_ROWS).min(last))
        .take(ROW_LIMIT)
}

fn load(
    paths: &CataloguePaths,
    reader: &ArtworkAccess,
    track: Candidate<'_>,
) -> Result<Option<Handle>, ReadFailure> {
    let catalogue = paths.open().map_err(|_| ReadFailure::Catalogue)?;
    let bytes = load_indexed(
        &ArtworkCache::new(paths.artwork_cache()),
        &catalogue,
        track.library,
        track.id,
        track.path,
        track.remote.then_some(reader),
    )?;
    Ok(bytes.as_deref().and_then(row_handle))
}

#[cfg(test)]
mod tests;
