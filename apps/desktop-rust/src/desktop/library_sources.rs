use std::{
    path::PathBuf,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
};

use iced::Task;
use notify::{RecommendedWatcher, RecursiveMode, Watcher};

use crate::catalogue::{
    Catalogue, CatalogueError, IndexedTrack, Library, MetadataOverride, Playlist, PlaylistEntry,
};

use super::Message;

#[derive(Clone)]
pub(crate) struct CataloguePaths {
    current: PathBuf,
    legacy: PathBuf,
}

impl CataloguePaths {
    pub(crate) fn in_directory(data: &std::path::Path) -> Self {
        Self {
            current: data.join("muzo-rust.sqlite"),
            legacy: data.join("muzo.sqlite"),
        }
    }

    pub(crate) fn from_environment() -> Option<Self> {
        let data = std::env::var_os("MUZO_DATA_DIR")
            .filter(|path| !path.is_empty())
            .map(PathBuf::from)
            .or_else(|| directories::BaseDirs::new().map(|dirs| dirs.data_dir().join("muzo")))?;
        Some(Self::in_directory(&data))
    }
}

#[derive(Debug, Clone)]
pub enum Operation {
    Startup,
    Add(PathBuf),
    Select(String),
    Refresh(String),
    Remove(String),
    Reconcile(Option<String>, Option<String>),
    CreatePlaylist(String),
    SelectPlaylist(String),
    AddToPlaylist(String, String),
    MovePlaylistEntry(String, String, i8),
    RemovePlaylistEntry(String, String),
    EditTrack(String, String, MetadataOverride),
    RecordPlays(Vec<String>, Option<String>, Option<String>),
}

#[derive(Debug, Clone)]
pub struct Snapshot {
    pub libraries: Vec<Library>,
    pub selected: Option<String>,
    pub tracks: Vec<IndexedTrack>,
    pub warnings: Vec<String>,
    pub playlists: Vec<Playlist>,
    pub selected_playlist: Option<String>,
    pub playlist_entries: Vec<PlaylistEntry>,
}

pub(crate) fn run(
    paths: &CataloguePaths,
    operation: Operation,
) -> Result<Snapshot, CatalogueError> {
    let mut catalogue = Catalogue::open(&paths.current, Some(&paths.legacy))?;
    let mut warnings = Vec::new();
    let mut selected_playlist = None;
    let selected = match operation {
        Operation::Add(path) => Some(catalogue.add_folder(&path)?.id),
        Operation::Select(id) => Some(id),
        Operation::Refresh(id) => {
            catalogue.refresh(&id)?;
            Some(id)
        }
        Operation::Remove(id) => {
            catalogue.remove(&id)?;
            None
        }
        Operation::CreatePlaylist(name) => {
            selected_playlist = Some(catalogue.create_playlist(&name)?.id);
            None
        }
        Operation::SelectPlaylist(id) => {
            selected_playlist = Some(id);
            None
        }
        Operation::AddToPlaylist(id, track) => {
            catalogue.add_to_playlist(&id, &track)?;
            selected_playlist = Some(id);
            None
        }
        Operation::MovePlaylistEntry(id, entry, delta) => {
            catalogue.move_playlist_entry(&id, &entry, delta)?;
            selected_playlist = Some(id);
            None
        }
        Operation::RemovePlaylistEntry(id, entry) => {
            catalogue.remove_playlist_entry(&id, &entry)?;
            selected_playlist = Some(id);
            None
        }
        Operation::EditTrack(id, library_id, edit) => {
            catalogue.override_track(&id, &edit)?;
            Some(library_id)
        }
        Operation::RecordPlays(ids, selection, playlist) => {
            for id in ids {
                catalogue.record_play(&id)?;
            }
            selected_playlist = playlist;
            selection
        }
        Operation::Startup | Operation::Reconcile(_, _) => {
            for source in catalogue
                .libraries()?
                .into_iter()
                .filter(|source| source.kind == "filesystem")
            {
                if let Err(error) = catalogue.refresh(&source.id) {
                    warnings.push(format!("{}: {error}", source.name));
                }
            }
            match operation {
                Operation::Reconcile(selected, playlist) => {
                    selected_playlist = playlist;
                    selected
                }
                _ => None,
            }
        }
    };
    let libraries = catalogue.libraries()?;
    let selected = selected
        .filter(|id| libraries.iter().any(|source| source.id == *id))
        .or_else(|| {
            libraries
                .iter()
                .find(|source| source.kind == "filesystem")
                .map(|source| source.id.clone())
        });
    let playlists = catalogue.playlists()?;
    let playlist_entries = selected_playlist
        .as_ref()
        .map(|id| catalogue.playlist_entries(id))
        .transpose()?
        .unwrap_or_default();
    let tracks = selected
        .as_ref()
        .map(|id| catalogue.tracks(id))
        .transpose()?
        .unwrap_or_default();
    Ok(Snapshot {
        libraries,
        selected,
        tracks,
        warnings,
        playlists,
        selected_playlist,
        playlist_entries,
    })
}

pub(super) struct Sources {
    paths: Option<CataloguePaths>,
    pub snapshot: Option<Snapshot>,
    pub search: String,
    pub pending_remove: Option<String>,
    pub error: Option<String>,
    pub playlist_name: String,
    pub editing: Option<(String, String, Box<[String; 7]>)>,
    pub queued_plays: Vec<String>,
    busy: bool,
    changed: Arc<AtomicBool>,
    watcher: Option<RecommendedWatcher>,
    roots: Vec<PathBuf>,
}

impl Sources {
    pub fn new(paths: Option<CataloguePaths>) -> Self {
        Self {
            paths,
            snapshot: None,
            search: String::new(),
            pending_remove: None,
            error: None,
            playlist_name: String::new(),
            editing: None,
            queued_plays: Vec::new(),
            busy: false,
            changed: Arc::new(AtomicBool::new(false)),
            watcher: None,
            roots: Vec::new(),
        }
    }

    pub fn busy(&self) -> bool {
        self.busy
    }

    pub fn start(&mut self, operation: Operation) -> Task<Message> {
        let Some(paths) = self.paths.clone().filter(|_| !self.busy) else {
            return Task::none();
        };
        self.busy = true;
        self.error = None;
        Task::perform(
            async move {
                tokio::task::spawn_blocking(move || {
                    run(&paths, operation).map_err(|error| error.to_string())
                })
                .await
                .map_err(|error| format!("Could not update the music catalogue: {error}"))?
            },
            Message::SourcesUpdated,
        )
    }

    pub fn apply(&mut self, result: Result<Snapshot, String>) {
        self.busy = false;
        let snapshot = match result {
            Ok(snapshot) => snapshot,
            Err(error) => {
                self.error = Some(error);
                return;
            }
        };
        self.error = snapshot.warnings.first().cloned();
        let roots: Vec<_> = snapshot
            .libraries
            .iter()
            .filter(|source| source.kind == "filesystem")
            .map(|source| source.location.clone())
            .collect();
        if self.roots != roots {
            self.install_watcher(&roots);
            self.roots = roots;
        }
        self.snapshot = Some(snapshot);
    }

    pub fn poll(&mut self) -> Task<Message> {
        if self.busy {
            return Task::none();
        }
        let selected = self
            .snapshot
            .as_ref()
            .and_then(|snapshot| snapshot.selected.clone());
        let playlist = self
            .snapshot
            .as_ref()
            .and_then(|snapshot| snapshot.selected_playlist.clone());
        if !self.queued_plays.is_empty() {
            let ids = std::mem::take(&mut self.queued_plays);
            return self.start(Operation::RecordPlays(ids, selected, playlist));
        }
        if !self.changed.swap(false, Ordering::AcqRel) {
            return Task::none();
        }
        self.start(Operation::Reconcile(selected, playlist))
    }

    fn install_watcher(&mut self, roots: &[PathBuf]) {
        let changed = Arc::clone(&self.changed);
        self.watcher =
            match notify::recommended_watcher(move |result: notify::Result<notify::Event>| {
                if result.as_ref().is_err() || result.is_ok_and(|event| !event.kind.is_access()) {
                    changed.store(true, Ordering::Release);
                }
            }) {
                Ok(mut watcher) => {
                    for root in roots.iter().filter(|root| root.is_dir()) {
                        if let Err(error) = watcher.watch(root, RecursiveMode::Recursive) {
                            self.error =
                                Some(format!("Could not watch {}: {error}", root.display()));
                        }
                    }
                    Some(watcher)
                }
                Err(error) => {
                    self.error = Some(error.to_string());
                    None
                }
            };
    }
}
