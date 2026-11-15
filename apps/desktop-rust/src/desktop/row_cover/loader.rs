use std::{collections::HashSet, ops::Range, sync::Arc};

use iced::{
    Task,
    widget::{image::Handle, scrollable::Viewport},
};
use lofty::{config::ParseOptions, probe::Probe};

use crate::{artwork_cache::ArtworkCache, catalogue::IndexedTrack, local_track::embedded_cover};

use super::super::{CataloguePaths, LibraryPage, Message, Snapshot};
use super::{ROW_LIMIT, remote::Remote, row_handle};

const CONCURRENT_READS: usize = 2;
const DEFAULT_ROWS: Range<usize> = 0..16;
const OVERSCAN_ROWS: usize = 4;

#[derive(Default)]
pub(in crate::desktop) struct Loader {
    generation: u64,
    visible: Option<(LibraryPage, Range<usize>)>,
    pending: HashSet<(u64, String)>,
    attempted: HashSet<String>,
    remote: Arc<Remote>,
}

#[derive(Debug, Clone)]
pub struct Complete {
    generation: u64,
    id: String,
    cover: Option<Handle>,
}

pub(in crate::desktop) fn viewport_rows(
    viewport: Viewport,
    row_height: f32,
    inset: f32,
) -> Range<usize> {
    let top = (viewport.absolute_offset().y - inset).max(0.0);
    let bottom = top + viewport.bounds().height;
    (top / row_height).floor() as usize..(bottom / row_height).ceil() as usize
}

impl Loader {
    pub(in crate::desktop) fn catalogue_changed(&mut self) {
        self.generation = self.generation.wrapping_add(1);
        self.attempted.clear();
    }

    pub(in crate::desktop) fn visible(&mut self, page: LibraryPage, rows: Range<usize>) {
        self.visible = Some((page, rows));
    }

    pub(in crate::desktop) fn complete(
        &mut self,
        result: Complete,
        snapshot: Option<&mut Snapshot>,
    ) {
        self.pending.remove(&(result.generation, result.id.clone()));
        if result.generation != self.generation {
            return;
        }
        if let (Some(snapshot), Some(cover)) = (snapshot, result.cover) {
            snapshot.covers.insert(result.id, cover);
        }
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
        let first = visible.start.min(last);
        let end = visible.end.min(last);
        let indices = (first..end)
            .chain(first.saturating_sub(OVERSCAN_ROWS)..first)
            .chain(end..end.saturating_add(OVERSCAN_ROWS).min(last));
        let candidates: Vec<_> = indices
            .filter_map(|index| {
                if page == LibraryPage::Songs {
                    snapshot.tracks.get(index)
                } else {
                    snapshot.playlist_entries.get(index)?.track.as_ref()
                }
            })
            .collect();
        if self.attempted.len() + candidates.len() > ROW_LIMIT {
            let visible_ids: HashSet<_> =
                candidates.iter().map(|track| track.id.as_str()).collect();
            self.attempted
                .retain(|id| visible_ids.contains(id.as_str()));
            snapshot
                .covers
                .retain(|id, _| visible_ids.contains(id.as_str()));
        }
        let mut requests = Vec::new();
        for track in candidates {
            if !self.attempted.insert(track.id.clone()) {
                continue;
            }
            let remote = snapshot
                .libraries
                .iter()
                .find(|source| source.id == track.library_id)
                .is_some_and(|source| source.kind == "dropbox");
            self.pending.insert((self.generation, track.id.clone()));
            requests.push(self.read(paths.clone(), track.clone(), remote));
            if requests.len() == available {
                break;
            }
        }
        Task::batch(requests)
    }

    fn read(&self, paths: CataloguePaths, track: IndexedTrack, remote: bool) -> Task<Message> {
        let reader = Arc::clone(&self.remote);
        let generation = self.generation;
        let id = track.id.clone();
        Task::perform(
            async move {
                let cover =
                    tokio::task::spawn_blocking(move || load(&paths, &reader, &track, remote))
                        .await
                        .ok()
                        .flatten();
                Complete {
                    generation,
                    id,
                    cover,
                }
            },
            Message::ArtworkReady,
        )
    }
}

fn load(
    paths: &CataloguePaths,
    reader: &Remote,
    track: &IndexedTrack,
    remote: bool,
) -> Option<Handle> {
    let catalogue = paths.open().ok()?;
    let revision = catalogue
        .artwork_revision(&track.library_id, &track.id)
        .ok()?;
    let cache = ArtworkCache::new(paths.artwork_cache());
    let bytes = cache.get_or_read(&revision, || {
        if remote {
            return reader
                .read(&catalogue, &track.library_id, &track.path.to_string_lossy())
                .ok()
                .flatten();
        }
        let file = Probe::open(&track.path)
            .ok()?
            .options(ParseOptions::new().read_properties(false))
            .read()
            .ok()?;
        embedded_cover(&file).map(|picture| picture.data().to_vec())
    })?;
    row_handle(&bytes)
}
