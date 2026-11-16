use std::{collections::HashMap, fmt, time::Duration};

use iced::{Task, task};

use crate::catalogue::IndexedTrack;

use super::{LibraryPage, Message, Snapshot, library_sources::CataloguePaths, row_cover};

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum SongScope {
    AllSongs,
    #[default]
    Source,
}

impl fmt::Display for SongScope {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::AllSongs => "All songs",
            Self::Source => "This source",
        })
    }
}

#[derive(Default)]
pub(super) struct TrackSearch {
    pub(super) scope: SongScope,
    all_songs: String,
    source: String,
    playlist: String,
    hits: Vec<IndexedTrack>,
    confirmed_since_request: HashMap<String, u32>,
    pub(super) projection: Option<Snapshot>,
    pub(super) loading: bool,
    pub(super) error: Option<String>,
    request: u64,
    pending: Option<task::Handle>,
}

impl TrackSearch {
    pub(super) fn song_scope(&self, snapshot: Option<&Snapshot>) -> SongScope {
        if snapshot.is_some_and(|snapshot| snapshot.selected.is_some()) {
            self.scope
        } else {
            SongScope::AllSongs
        }
    }

    pub(super) fn query(&self, page: LibraryPage, snapshot: Option<&Snapshot>) -> &str {
        match page {
            LibraryPage::Songs => match self.song_scope(snapshot) {
                SongScope::AllSongs => &self.all_songs,
                SongScope::Source => &self.source,
            },
            LibraryPage::Playlists => &self.playlist,
            LibraryPage::Settings => "",
        }
    }

    pub(super) fn change(
        &mut self,
        page: LibraryPage,
        snapshot: Option<&Snapshot>,
        query: String,
        paths: Option<CataloguePaths>,
    ) -> Task<Message> {
        let refresh = match page {
            LibraryPage::Songs => match self.song_scope(snapshot) {
                SongScope::Source => {
                    self.source = query;
                    Task::none()
                }
                SongScope::AllSongs => {
                    self.all_songs = query;
                    self.hits.clear();
                    self.refresh_global(page, snapshot, paths)
                }
            },
            LibraryPage::Playlists => {
                self.playlist = query;
                Task::none()
            }
            LibraryPage::Settings => Task::none(),
        };
        self.project(page, snapshot);
        refresh
    }

    pub(super) fn selected_source(&mut self, selected: bool) {
        self.scope = if selected {
            SongScope::Source
        } else {
            SongScope::AllSongs
        };
        self.source.clear();
    }

    pub(super) fn clear_playlist(&mut self) {
        self.playlist.clear();
    }

    pub(super) fn uses_global(&self, page: LibraryPage, snapshot: Option<&Snapshot>) -> bool {
        page == LibraryPage::Songs
            && self.song_scope(snapshot) == SongScope::AllSongs
            && !self.all_songs.trim().is_empty()
    }

    pub(super) fn confirm_play_count(&mut self, id: &str, count: u32) {
        if self.loading {
            let recorded = self.confirmed_since_request.entry(id.into()).or_default();
            *recorded = (*recorded).max(count);
        }
        if let Some(track) = self.hits.iter_mut().find(|track| track.id == id) {
            track.plays = track.plays.max(count);
        }
    }

    pub(super) fn refresh_global(
        &mut self,
        page: LibraryPage,
        snapshot: Option<&Snapshot>,
        paths: Option<CataloguePaths>,
    ) -> Task<Message> {
        self.confirmed_since_request.clear();
        let Some(paths) = paths.filter(|_| self.uses_global(page, snapshot)) else {
            self.pending = None;
            self.request = self.request.wrapping_add(1);
            self.loading = false;
            self.error = None;
            return Task::none();
        };
        const SEARCH_DEBOUNCE: Duration = Duration::from_millis(130);
        self.request = self.request.wrapping_add(1);
        let serial = self.request;
        let query = Terms::new(&self.all_songs);
        self.error = None;
        self.loading = true;
        let (task, handle) = Task::perform(
            async move {
                tokio::time::sleep(SEARCH_DEBOUNCE).await;
                tokio::task::spawn_blocking(move || {
                    let catalogue = paths.open()?;
                    let tracks = catalogue
                        .all_indexed_tracks()?
                        .into_iter()
                        .filter(|track| query.matches(track))
                        .collect();
                    Ok::<_, crate::catalogue::CatalogueError>(tracks)
                })
                .await
                .map_err(|error| error.to_string())?
                .map_err(|error| error.to_string())
            },
            move |result| Message::TrackSearchFinished(serial, result),
        )
        .abortable();
        self.pending = Some(handle.abort_on_drop());
        task
    }

    pub(super) fn finish(
        &mut self,
        serial: u64,
        result: Result<Vec<IndexedTrack>, String>,
    ) -> bool {
        if serial != self.request {
            return false;
        }
        self.pending = None;
        self.loading = false;
        match result {
            Ok(mut tracks) => {
                for track in &mut tracks {
                    if let Some(&confirmed) = self.confirmed_since_request.get(&track.id) {
                        track.plays = track.plays.max(confirmed);
                    }
                }
                self.hits = tracks;
            }
            Err(error) => self.error = Some(error),
        }
        self.confirmed_since_request.clear();
        true
    }

    pub(super) fn global_hit(&self, library: &str, id: &str) -> bool {
        self.projection.as_ref().is_some_and(|snapshot| {
            self.uses_global(LibraryPage::Songs, Some(snapshot))
                && snapshot
                    .tracks
                    .iter()
                    .any(|track| track.library_id == library && track.id == id)
        })
    }

    pub(super) fn project(&mut self, page: LibraryPage, snapshot: Option<&Snapshot>) {
        let Some(snapshot) =
            snapshot.filter(|snapshot| !self.query(page, Some(snapshot)).trim().is_empty())
        else {
            self.projection = None;
            return;
        };
        let terms = Terms::new(self.query(page, Some(snapshot)));
        let mut projection = snapshot.clone();
        match page {
            LibraryPage::Songs if self.uses_global(page, Some(snapshot)) => {
                projection.tracks.clone_from(&self.hits);
                projection.tracks.retain(|track| {
                    snapshot
                        .libraries
                        .iter()
                        .any(|library| library.id == track.library_id)
                });
            }
            LibraryPage::Songs => projection.tracks.retain(|track| terms.matches(track)),
            LibraryPage::Playlists => projection.playlist_entries.retain(|entry| {
                entry
                    .track
                    .as_ref()
                    .is_some_and(|track| terms.matches(track))
            }),
            LibraryPage::Settings => return,
        }
        if let Some(previous) = &self.projection {
            row_cover::retain_unchanged(previous, &mut projection);
        }
        self.projection = Some(projection);
    }
}

struct Terms(Vec<String>);

impl Terms {
    fn new(query: &str) -> Self {
        Self(query.split_whitespace().map(str::to_lowercase).collect())
    }

    fn matches(&self, track: &IndexedTrack) -> bool {
        let title = track.title.to_lowercase();
        let artist = track.artist.to_lowercase();
        let album = track.album.as_deref().unwrap_or_default().to_lowercase();
        self.0
            .iter()
            .all(|term| title.contains(term) || artist.contains(term) || album.contains(term))
    }
}
