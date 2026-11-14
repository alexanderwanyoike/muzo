use std::{
    path::PathBuf,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
};

use iced::Task;
use notify::{
    EventKind, RecommendedWatcher, RecursiveMode, Watcher,
    event::{ModifyKind, RenameMode},
};

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

    pub(crate) fn open(&self) -> Result<Catalogue, CatalogueError> {
        Catalogue::open(&self.current, Some(&self.legacy))
    }

    pub(crate) fn update_cache(&self) -> PathBuf {
        self.current.with_file_name("updates-rust")
    }

    pub(crate) fn artwork_cache(&self) -> PathBuf {
        self.current.with_file_name("artwork-rust")
    }

    pub(crate) fn legacy_appearance() -> Option<PathBuf> {
        let profile = directories::BaseDirs::new()?;
        Some(
            profile
                .config_dir()
                .join("Muzo")
                .join("Local Storage")
                .join("leveldb"),
        )
    }

    pub(crate) fn sidebar(&self) -> PathBuf {
        self.current.with_file_name("sidebar-rust.json")
    }

    pub(crate) fn appearance(&self) -> PathBuf {
        self.current.with_file_name("appearance-rust.json")
    }

    pub(crate) fn dropbox_cache(&self) -> PathBuf {
        self.current.with_file_name("dropbox-audio-rust")
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
    LoadTrackEditor(String, String),
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
    pub editor: Option<(String, String, Box<[String; 7]>)>,
}

pub(crate) fn run(
    paths: &CataloguePaths,
    operation: Operation,
) -> Result<Snapshot, CatalogueError> {
    let mut catalogue = paths.open()?;
    let mut warnings = Vec::new();
    let mut selected_playlist = None;
    let mut editor = None;
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
        Operation::LoadTrackEditor(id, library_id) => {
            let edit = catalogue.track_overrides(&id)?;
            let field =
                |value: Option<u32>| value.map_or_else(String::new, |value| value.to_string());
            editor = Some((
                id,
                library_id.clone(),
                Box::new([
                    edit.title.unwrap_or_default(),
                    edit.artist.unwrap_or_default(),
                    edit.album.unwrap_or_default(),
                    field(edit.track_number),
                    field(edit.disc_number),
                    edit.genre.unwrap_or_default(),
                    field(edit.year),
                ]),
            ));
            Some(library_id)
        }
        Operation::EditTrack(id, library_id, edit) => {
            catalogue.override_track(&id, &edit)?;
            Some(library_id)
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
        editor,
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
    pub saving_editor: bool,
    busy: bool,
    changed: Arc<AtomicBool>,
    roots_changed: Arc<AtomicBool>,
    watcher: Option<RecommendedWatcher>,
    roots: Vec<PathBuf>,
    registered: Vec<PathBuf>,
    pending: std::collections::VecDeque<Operation>,
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
            saving_editor: false,
            busy: false,
            changed: Arc::new(AtomicBool::new(false)),
            roots_changed: Arc::new(AtomicBool::new(false)),
            watcher: None,
            roots: Vec::new(),
            registered: Vec::new(),
            pending: std::collections::VecDeque::new(),
        }
    }

    pub(super) fn paths(&self) -> Option<CataloguePaths> {
        self.paths.clone()
    }

    pub(super) fn reload(&mut self) -> Task<Message> {
        let selected = self
            .snapshot
            .as_ref()
            .and_then(|snapshot| snapshot.selected.clone());
        self.changed.store(true, Ordering::Release);
        selected.map_or_else(Task::none, |id| self.start(Operation::Select(id)))
    }

    pub fn confirm_play_count(&mut self, id: &str, count: u32) {
        if let Some(snapshot) = &mut self.snapshot {
            for track in &mut snapshot.tracks {
                if track.id == id {
                    track.plays = track.plays.max(count);
                }
            }
            for entry in &mut snapshot.playlist_entries {
                if let Some(track) = &mut entry.track
                    && track.id == id
                {
                    track.plays = track.plays.max(count);
                }
            }
        }
    }

    pub fn busy(&self) -> bool {
        self.busy
    }

    pub fn start(&mut self, operation: Operation) -> Task<Message> {
        if self.busy || !self.pending.is_empty() {
            self.pending.push_back(operation);
            return Task::none();
        }
        self.run_operation(operation)
    }

    fn run_operation(&mut self, operation: Operation) -> Task<Message> {
        let Some(paths) = self.paths.clone() else {
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
        let mut snapshot = match result {
            Ok(snapshot) => snapshot,
            Err(error) => {
                self.saving_editor = false;
                self.error = Some(error);
                return;
            }
        };
        if let Some(previous) = &self.snapshot {
            preserve_play_counts(&mut snapshot, previous);
        }
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
        } else {
            self.retry_watch_roots();
        }
        if let Some(fields) = snapshot.editor.clone() {
            self.editing = Some(fields);
        }
        if self.saving_editor {
            self.editing = None;
        }
        self.saving_editor = false;
        self.snapshot = Some(snapshot);
    }

    pub fn poll(&mut self) -> Task<Message> {
        if self.busy {
            return Task::none();
        }
        self.retry_watch_roots();
        if let Some(operation) = self.pending.pop_front() {
            return self.run_operation(operation);
        }
        let selected = self
            .snapshot
            .as_ref()
            .and_then(|snapshot| snapshot.selected.clone());
        let playlist = self
            .snapshot
            .as_ref()
            .and_then(|snapshot| snapshot.selected_playlist.clone());
        if !self.changed.swap(false, Ordering::AcqRel) {
            return Task::none();
        }
        self.start(Operation::Reconcile(selected, playlist))
    }

    fn install_watcher(&mut self, roots: &[PathBuf]) {
        self.registered.clear();
        let changed = Arc::clone(&self.changed);
        let roots_changed = Arc::clone(&self.roots_changed);
        let watched_paths = roots.to_vec();
        self.watcher =
            match notify::recommended_watcher(move |result: notify::Result<notify::Event>| {
                let touches_root = result.as_ref().is_ok_and(|event| {
                    matches!(
                        event.kind,
                        EventKind::Remove(_)
                            | EventKind::Modify(ModifyKind::Name(
                                RenameMode::From | RenameMode::Both | RenameMode::Any
                            ))
                    ) && event.paths.iter().any(|path| watched_paths.contains(path))
                });
                if result.is_err() || touches_root {
                    roots_changed.store(true, Ordering::Release);
                }
                if result.is_err() || result.is_ok_and(|event| !event.kind.is_access()) {
                    changed.store(true, Ordering::Release);
                }
            }) {
                Ok(watcher) => Some(watcher),
                Err(error) => {
                    self.error = Some(error.to_string());
                    None
                }
            };
        self.register_available(roots);
    }

    fn retry_watch_roots(&mut self) {
        let roots = self.roots.clone();
        if self.roots_changed.swap(false, Ordering::AcqRel)
            || self.registered.iter().any(|root| !root.is_dir())
            || (self.watcher.is_none() && !roots.is_empty())
        {
            self.install_watcher(&roots);
        } else {
            self.register_available(&roots);
        }
    }

    fn register_available(&mut self, roots: &[PathBuf]) {
        let Some(watcher) = &mut self.watcher else {
            return;
        };
        for root in roots {
            if self.registered.contains(root) || !root.is_dir() {
                continue;
            }
            match watcher.watch(root, RecursiveMode::Recursive) {
                Ok(()) => {
                    self.registered.push(root.clone());
                    self.changed.store(true, Ordering::Release);
                }
                Err(error) => {
                    self.error = Some(format!("Could not watch {}: {error}", root.display()))
                }
            }
        }
    }
}

fn preserve_play_counts(snapshot: &mut Snapshot, previous: &Snapshot) {
    let previous_counts: std::collections::HashMap<_, _> = previous
        .tracks
        .iter()
        .chain(
            previous
                .playlist_entries
                .iter()
                .filter_map(|entry| entry.track.as_ref()),
        )
        .map(|track| (track.id.as_str(), track.plays))
        .collect();
    for track in snapshot.tracks.iter_mut().chain(
        snapshot
            .playlist_entries
            .iter_mut()
            .filter_map(|entry| entry.track.as_mut()),
    ) {
        if let Some(previous) = previous_counts.get(track.id.as_str()) {
            track.plays = track.plays.max(*previous);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_starts_queued_source_actions_before_new_arrivals() {
        let root = tempfile::tempdir().unwrap();
        let mut sources = Sources::new(Some(CataloguePaths::in_directory(root.path())));
        assert!(sources.start(Operation::Select("first".into())).units() > 0);
        assert_eq!(sources.start(Operation::Select("second".into())).units(), 0);

        sources.apply(Err("first finished".into()));

        assert_eq!(sources.start(Operation::Select("third".into())).units(), 0);
        assert!(sources.poll().units() > 0);
        assert!(matches!(sources.pending.front(), Some(Operation::Select(id)) if id == "third"));
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn it_reattaches_a_watched_folder_replaced_between_polls() {
        let root = tempfile::tempdir().unwrap();
        let library = root.path().join("library");
        std::fs::create_dir(&library).unwrap();
        let mut sources = Sources::new(None);
        sources.roots = vec![library.clone()];
        sources.install_watcher(std::slice::from_ref(&library));
        sources.changed.store(false, Ordering::Release);

        std::fs::rename(&library, root.path().join("renamed")).unwrap();
        std::fs::create_dir(&library).unwrap();
        wait_for_event(&sources.changed);
        let _ = sources.poll();
        sources.changed.store(false, Ordering::Release);

        std::fs::write(library.join("Track.mp3"), "fixture").unwrap();
        wait_for_event(&sources.changed);
    }

    #[test]
    fn it_drops_a_missing_root_registration_without_a_notification() {
        let root = tempfile::tempdir().unwrap();
        let missing = root.path().join("library");
        let mut sources = Sources::new(None);
        sources.roots = vec![missing.clone()];
        sources.watcher =
            Some(notify::recommended_watcher(|_: notify::Result<notify::Event>| {}).unwrap());
        sources.registered = vec![missing];

        let _ = sources.poll();

        assert!(sources.registered.is_empty());
    }

    #[cfg(target_os = "linux")]
    fn wait_for_event(changed: &AtomicBool) {
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(2);
        while !changed.load(Ordering::Acquire) {
            assert!(std::time::Instant::now() < deadline, "no filesystem event");
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
    }
}

#[cfg(test)]
mod navigation_tests {
    use super::*;

    #[test]
    fn it_keeps_the_selected_catalogue_during_a_background_reload() {
        let directory = tempfile::tempdir().unwrap();
        let mut sources = Sources::new(Some(CataloguePaths::in_directory(directory.path())));
        sources.snapshot = Some(Snapshot {
            libraries: vec![],
            selected: Some("picked-source".into()),
            tracks: vec![],
            warnings: vec![],
            playlists: vec![],
            selected_playlist: Some("picked-playlist".into()),
            playlist_entries: vec![],
            editor: None,
        });
        assert_eq!(sources.reload().units(), 0);
        assert!(sources.changed.load(Ordering::Acquire));
    }
}
