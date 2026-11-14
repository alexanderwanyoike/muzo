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
    Reload,
    Add(PathBuf),
    AddNamedFolder(String, PathBuf),
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
    pub track_counts: std::collections::HashMap<String, u64>,
    pub playlist_counts: std::collections::HashMap<String, u64>,
    pub covers: super::row_cover::Covers,
    pub warnings: Vec<String>,
    pub playlists: Vec<Playlist>,
    pub selected_playlist: Option<String>,
    pub playlist_entries: Vec<PlaylistEntry>,
    pub editor: Option<(String, String, Box<[String; 7]>)>,
}

#[cfg(test)]
pub(crate) fn run(
    paths: &CataloguePaths,
    operation: Operation,
) -> Result<Snapshot, CatalogueError> {
    run_with_selection(paths, operation, None, None)
}

fn run_with_selection(
    paths: &CataloguePaths,
    operation: Operation,
    previous_source: Option<String>,
    previous_playlist: Option<String>,
) -> Result<Snapshot, CatalogueError> {
    let mut catalogue = paths.open()?;
    let mut warnings = Vec::new();
    let mut selected_playlist = previous_playlist;
    let mut editor = None;
    let selected = match operation {
        Operation::Add(path) => Some(catalogue.add_folder(&path)?.id),
        Operation::AddNamedFolder(name, path) => Some(catalogue.add_named_folder(&path, &name)?.id),
        Operation::Select(id) => Some(id),
        Operation::Refresh(id) => {
            catalogue.refresh(&id)?;
            Some(id)
        }
        Operation::Remove(id) => {
            catalogue.remove(&id)?;
            previous_source.clone()
        }
        Operation::CreatePlaylist(name) => {
            let transaction = catalogue.begin_write()?;
            let playlist = catalogue.create_playlist(&name)?;
            let mut snapshot = read_snapshot(
                &catalogue,
                previous_source,
                Some(playlist.id),
                None,
                warnings,
            )?;
            transaction.commit()?;
            load_covers(paths, &catalogue, &mut snapshot);
            return Ok(snapshot);
        }
        Operation::SelectPlaylist(id) => {
            selected_playlist = Some(id);
            previous_source.clone()
        }
        Operation::AddToPlaylist(id, track) => {
            catalogue.add_to_playlist(&id, &track)?;
            selected_playlist = selected_playlist.or(Some(id));
            previous_source.clone()
        }
        Operation::MovePlaylistEntry(id, entry, delta) => {
            catalogue.move_playlist_entry(&id, &entry, delta)?;
            selected_playlist = Some(id);
            previous_source.clone()
        }
        Operation::RemovePlaylistEntry(id, entry) => {
            catalogue.remove_playlist_entry(&id, &entry)?;
            selected_playlist = Some(id);
            previous_source.clone()
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
        Operation::Reload => previous_source.clone(),
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
    let mut snapshot = read_snapshot(
        &catalogue,
        selected.or(previous_source),
        selected_playlist,
        editor,
        warnings,
    )?;
    load_covers(paths, &catalogue, &mut snapshot);
    Ok(snapshot)
}

fn read_snapshot(
    catalogue: &Catalogue,
    selected: Option<String>,
    selected_playlist: Option<String>,
    editor: Option<(String, String, Box<[String; 7]>)>,
    warnings: Vec<String>,
) -> Result<Snapshot, CatalogueError> {
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
    let selected_playlist = selected_playlist
        .filter(|id| playlists.iter().any(|item| item.id == *id))
        .or_else(|| playlists.first().map(|item| item.id.clone()));
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
        track_counts: catalogue.track_counts()?,
        playlist_counts: catalogue.playlist_counts()?,
        covers: Default::default(),
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

fn load_covers(paths: &CataloguePaths, catalogue: &Catalogue, snapshot: &mut Snapshot) {
    let playlist_covers = super::row_cover::indexed(
        paths,
        catalogue,
        &snapshot.libraries,
        snapshot
            .playlist_entries
            .iter()
            .filter_map(|entry| entry.track.as_ref()),
    );
    snapshot.covers =
        super::row_cover::indexed(paths, catalogue, &snapshot.libraries, &snapshot.tracks);
    snapshot.covers.extend(playlist_covers);
}

pub(super) struct Sources {
    paths: Option<CataloguePaths>,
    pub snapshot: Option<Snapshot>,
    pub search: String,
    pub pending_remove: Option<String>,
    pub error: Option<String>,
    pub playlist_name: String,
    pub playlist_search: String,
    pub show_playlist_form: bool,
    pub folder_name: String,
    pub folder_location: String,
    adding_folder: bool,
    creating_playlist: bool,
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
            playlist_search: String::new(),
            show_playlist_form: false,
            folder_name: String::new(),
            folder_location: String::new(),
            adding_folder: false,
            creating_playlist: false,
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
        self.start(Operation::Reload)
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

    pub fn playlist_save_pending(&self) -> bool {
        self.creating_playlist
            || self
                .pending
                .iter()
                .any(|operation| matches!(operation, Operation::CreatePlaylist(_)))
    }

    pub fn folder_save_pending(&self) -> bool {
        self.adding_folder
            || self
                .pending
                .iter()
                .any(|operation| matches!(operation, Operation::AddNamedFolder(_, _)))
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
        self.creating_playlist = matches!(operation, Operation::CreatePlaylist(_));
        self.adding_folder = matches!(operation, Operation::AddNamedFolder(_, _));
        self.error = None;
        let selected = self
            .snapshot
            .as_ref()
            .and_then(|snapshot| snapshot.selected.clone());
        let playlist = self
            .snapshot
            .as_ref()
            .and_then(|snapshot| snapshot.selected_playlist.clone());
        Task::perform(
            async move {
                tokio::task::spawn_blocking(move || {
                    run_with_selection(&paths, operation, selected, playlist)
                        .map_err(|error| error.to_string())
                })
                .await
                .map_err(|error| format!("Could not update the music catalogue: {error}"))?
            },
            Message::SourcesUpdated,
        )
    }

    pub fn apply(&mut self, result: Result<Snapshot, String>) {
        self.busy = false;
        let creating_playlist = std::mem::take(&mut self.creating_playlist);
        let adding_folder = std::mem::take(&mut self.adding_folder);
        let mut snapshot = match result {
            Ok(snapshot) => snapshot,
            Err(error) => {
                self.saving_editor = false;
                self.error = Some(error);
                return;
            }
        };
        if creating_playlist {
            self.show_playlist_form = false;
            self.playlist_name.clear();
            self.playlist_search.clear();
        }
        if adding_folder {
            self.folder_name.clear();
            self.folder_location.clear();
        }
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
    fn it_rolls_back_a_playlist_when_its_catalogue_snapshot_cannot_be_read() {
        let root = tempfile::tempdir().unwrap();
        let paths = CataloguePaths::in_directory(root.path());
        let catalogue = paths.open().unwrap();
        let connection = rusqlite::Connection::open(&paths.current).unwrap();
        connection.execute(
            "INSERT INTO libraries (id, name, kind, location) VALUES ('broken', 'Test', 'filesystem', '/test')",
            [],
        ).unwrap();
        connection.execute(
            "INSERT INTO tracks (id, library_id, title, artist, duration_seconds, file_path, file_size, file_mtime) VALUES ('bad', 'broken', 'Test', '', 'invalid', '/test/bad.mp3', 0, 0)",
            [],
        ).unwrap();

        let result = run(&paths, Operation::CreatePlaylist("Evening".into()));

        assert!(result.is_err());
        assert!(catalogue.playlists().unwrap().is_empty());
        connection
            .execute("UPDATE tracks SET duration_seconds=1", [])
            .unwrap();
        let retry = run(&paths, Operation::CreatePlaylist("Evening".into())).unwrap();
        assert_eq!(retry.playlists.len(), 1);
        assert_eq!(retry.playlists[0].name, "Evening");
    }

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
    fn it_keeps_the_selected_source_and_playlist_when_adding_a_song_elsewhere() {
        let directory = tempfile::tempdir().unwrap();
        let paths = CataloguePaths::in_directory(directory.path());
        let mut catalogue = paths.open().unwrap();
        let first = directory.path().join("first");
        let second = directory.path().join("second");
        std::fs::create_dir(&first).unwrap();
        std::fs::create_dir(&second).unwrap();
        std::fs::copy(
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/tone.mp3"),
            second.join("tone.mp3"),
        )
        .unwrap();
        catalogue.add_folder(&first).unwrap();
        let source = catalogue.add_folder(&second).unwrap().id;
        let track = catalogue.tracks(&source).unwrap().remove(0).id;
        let selected = catalogue.create_playlist("Selected").unwrap().id;
        let target = catalogue.create_playlist("Target").unwrap().id;

        let snapshot = run_with_selection(
            &paths,
            Operation::AddToPlaylist(target.clone(), track),
            Some(source.clone()),
            Some(selected.clone()),
        )
        .unwrap();

        assert_eq!(snapshot.selected, Some(source));
        assert_eq!(snapshot.selected_playlist, Some(selected));
        assert_eq!(catalogue.playlist_entries(&target).unwrap().len(), 1);
    }
}

#[cfg(test)]
mod artwork_tests {
    use super::*;

    #[test]
    fn it_prepares_bounded_cover_art_for_an_unplayed_catalogued_song() {
        let directory = tempfile::tempdir().unwrap();
        let source = directory.path().join("songs");
        std::fs::create_dir(&source).unwrap();
        std::fs::copy(
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/embedded-cover.mp3"),
            source.join("covered.mp3"),
        )
        .unwrap();
        let paths = CataloguePaths::in_directory(directory.path());

        let snapshot = run(&paths, Operation::Add(source)).unwrap();
        let row = &snapshot.tracks[0];
        assert!(snapshot.covers.contains_key(&row.id));
        assert_eq!(row.plays, 0);
    }
}

#[cfg(test)]
mod count_tests {
    use super::*;

    #[test]
    fn it_reports_track_counts_for_selected_and_unselected_sources() {
        let directory = tempfile::tempdir().unwrap();
        let paths = CataloguePaths::in_directory(directory.path());
        let fixture = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/tone.mp3");
        let first = directory.path().join("first");
        let second = directory.path().join("second");
        for folder in [&first, &second] {
            std::fs::create_dir(folder).unwrap();
            std::fs::copy(&fixture, folder.join("tone.mp3")).unwrap();
            run(&paths, Operation::Add(folder.clone())).unwrap();
        }
        let snapshot = run(&paths, Operation::Startup).unwrap();
        assert_eq!(snapshot.tracks.len(), 1);
        assert_eq!(snapshot.track_counts.values().sum::<u64>(), 2);
        assert_eq!(snapshot.track_counts.len(), 2);
    }
}
