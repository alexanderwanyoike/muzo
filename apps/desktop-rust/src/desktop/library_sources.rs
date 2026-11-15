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

use super::{Message, track_editor_focus::Control};

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
    pub editor: Option<Box<TrackEditor>>,
}

#[derive(Debug, Clone)]
pub struct TrackEditor {
    pub track_id: String,
    pub library_id: String,
    pub fields: [String; 7],
    pub overridden: bool,
    pub error: Option<String>,
    pub action_focus: Option<Control>,
}

#[derive(Debug)]
pub(super) struct PlaylistPicker {
    pub track_id: String,
    pub library_id: String,
    pub title: String,
    pub playlist: Option<String>,
    pub error: Option<String>,
    pub focus: Option<super::playlist_picker_focus::Control>,
    pub preview: Option<String>,
}

struct PlaylistAddition {
    playlist: String,
    track: String,
    title: String,
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
        Operation::AddNamedFolder(name, path) => {
            let mut snapshot =
                catalogue.add_named_folder_with_snapshot(&path, &name, |catalogue, library| {
                    read_snapshot(
                        catalogue,
                        Some(library.id),
                        selected_playlist,
                        None,
                        warnings,
                    )
                })?;
            load_covers(paths, &catalogue, &mut snapshot);
            return Ok(snapshot);
        }
        Operation::Select(id) => Some(id),
        Operation::Refresh(id) => {
            catalogue.refresh(&id)?;
            None
        }
        Operation::Remove(id) => {
            let mut snapshot = catalogue.remove_with_snapshot(&id, |catalogue| {
                read_snapshot(
                    catalogue,
                    previous_source,
                    selected_playlist,
                    None,
                    warnings,
                )
            })?;
            load_covers(paths, &catalogue, &mut snapshot);
            return Ok(snapshot);
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
        Operation::AddToPlaylist(playlist, track) => {
            let transaction = catalogue.begin_write()?;
            catalogue.insert_playlist_entry(&playlist, &track)?;
            let mut snapshot = read_snapshot(
                &catalogue,
                previous_source,
                selected_playlist.or(Some(playlist)),
                None,
                warnings,
            )?;
            transaction.commit()?;
            load_covers(paths, &catalogue, &mut snapshot);
            return Ok(snapshot);
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
            let metadata = catalogue.track_metadata(&id, &library_id)?;
            editor = Some(Box::new(TrackEditor {
                track_id: id,
                library_id: library_id.clone(),
                fields: [
                    metadata.title,
                    metadata.artist,
                    metadata.album.unwrap_or_default(),
                    metadata.track_number.unwrap_or_default(),
                    metadata.disc_number.unwrap_or_default(),
                    metadata.genre.unwrap_or_default(),
                    metadata.year.unwrap_or_default(),
                ],
                overridden: metadata.overridden,
                error: None,
                action_focus: None,
            }));
            previous_source.clone().or(Some(library_id))
        }
        Operation::EditTrack(id, library_id, edit) => {
            let transaction = catalogue.begin_write()?;
            catalogue.override_library_track(&id, &library_id, &edit)?;
            let mut snapshot = read_snapshot(
                &catalogue,
                previous_source.or(Some(library_id)),
                selected_playlist,
                None,
                warnings,
            )?;
            transaction.commit()?;
            load_covers(paths, &catalogue, &mut snapshot);
            return Ok(snapshot);
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
    editor: Option<Box<TrackEditor>>,
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
    pub choosing_folder: bool,
    adding_folder: bool,
    creating_playlist: bool,
    pub editing: Option<Box<TrackEditor>>,
    pub playlist_picker: Option<PlaylistPicker>,
    pub playlist_message: Option<String>,
    last_playlist_choice: Option<String>,
    adding_track: Option<PlaylistAddition>,
    pending_editor: Option<(String, String)>,
    loading_editor: bool,
    saving_editor: bool,
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
            choosing_folder: false,
            adding_folder: false,
            creating_playlist: false,
            editing: None,
            playlist_picker: None,
            playlist_message: None,
            last_playlist_choice: None,
            adding_track: None,
            pending_editor: None,
            loading_editor: false,
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

    pub fn editor_save_pending(&self) -> bool {
        self.saving_editor
            || self
                .pending
                .iter()
                .any(|operation| matches!(operation, Operation::EditTrack(..)))
    }

    pub fn open_editor(&mut self, track: IndexedTrack) -> Task<Message> {
        if self.busy() || self.editor_save_pending() {
            return Task::none();
        }
        self.close_editor();
        self.pending_editor = Some((track.id.clone(), track.library_id.clone()));
        self.start(Operation::LoadTrackEditor(track.id, track.library_id))
    }

    pub fn close_editor(&mut self) {
        self.pending_editor = None;
        self.editing = None;
        self.pending
            .retain(|operation| !matches!(operation, Operation::LoadTrackEditor(..)));
    }

    pub fn open_playlist_picker(&mut self, track: IndexedTrack) {
        if self.busy() || self.track_add_pending() || self.editor_save_pending() {
            return;
        }
        self.close_editor();
        self.playlist_message = None;
        let playlist = self.snapshot.as_ref().and_then(|snapshot| {
            snapshot
                .playlists
                .iter()
                .find(|playlist| Some(&playlist.id) == self.last_playlist_choice.as_ref())
                .or_else(|| snapshot.playlists.first())
        });
        self.playlist_picker = Some(PlaylistPicker {
            track_id: track.id,
            library_id: track.library_id,
            title: track.title,
            playlist: playlist.map(|playlist| playlist.id.clone()),
            error: None,
            focus: None,
            preview: None,
        });
    }

    pub fn choose_track_playlist(&mut self, id: String) {
        if !self.track_add_pending()
            && let Some(draft) = self.playlist_picker.as_mut()
        {
            self.last_playlist_choice = Some(id.clone());
            draft.playlist = Some(id);
            draft.preview = None;
            draft.error = None;
        }
    }

    pub fn step_track_playlist(&mut self, step: i8) -> Option<String> {
        if self.track_add_pending() {
            return None;
        }
        let draft = self.playlist_picker.as_mut()?;
        let playlists = &self.snapshot.as_ref()?.playlists;
        let current = draft.preview.as_ref().or(draft.playlist.as_ref())?;
        let current = playlists
            .iter()
            .position(|playlist| &playlist.id == current)?;
        let index = current.checked_add_signed(isize::from(step))?;
        let next = playlists.get(index)?.id.clone();
        if draft.preview.is_some() {
            draft.preview = Some(next.clone());
            Some(next)
        } else {
            self.choose_track_playlist(next);
            None
        }
    }

    pub fn close_track_panels(&mut self) {
        self.close_editor();
        self.playlist_picker = None;
        self.playlist_message = None;
        self.last_playlist_choice = None;
    }

    pub fn track_add_pending(&self) -> bool {
        self.adding_track.is_some()
            || self
                .pending
                .iter()
                .any(|operation| matches!(operation, Operation::AddToPlaylist(..)))
    }

    pub fn submit_playlist_picker(&mut self) -> Task<Message> {
        if self.busy() || self.track_add_pending() {
            return Task::none();
        }
        let Some(picker) = self.playlist_picker.as_mut() else {
            return Task::none();
        };
        let Some(playlist) = picker.playlist.clone() else {
            return Task::none();
        };
        picker.error = None;
        picker.preview = None;
        let operation = Operation::AddToPlaylist(playlist, picker.track_id.clone());
        self.start(operation)
    }

    pub fn folder_editable(&self) -> bool {
        !self.choosing_folder && !self.folder_save_pending()
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
        if let Operation::AddToPlaylist(playlist, track) = &operation
            && let Some(picker) = self
                .playlist_picker
                .as_ref()
                .filter(|picker| picker.track_id == *track)
        {
            self.adding_track = Some(PlaylistAddition {
                playlist: playlist.clone(),
                track: track.clone(),
                title: picker.title.clone(),
            });
        }
        self.loading_editor = matches!(operation, Operation::LoadTrackEditor(..));
        self.saving_editor = matches!(operation, Operation::EditTrack(..));
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
        let saving_editor = std::mem::take(&mut self.saving_editor);
        let loading_editor = std::mem::take(&mut self.loading_editor);
        let adding_track = self.adding_track.take();
        let mut snapshot = match result {
            Ok(snapshot) => snapshot,
            Err(error) => {
                if let Some(adding_track) = adding_track {
                    match self
                        .playlist_picker
                        .as_mut()
                        .filter(|draft| draft.track_id == adding_track.track)
                    {
                        Some(draft) => draft.error = Some(error),
                        None => self.error = Some(error),
                    }
                } else if saving_editor {
                    if let Some(editor) = &mut self.editing {
                        editor.error = Some(error);
                    } else {
                        self.error = Some(error);
                    }
                } else if !loading_editor || self.pending_editor.take().is_some() {
                    self.error = Some(error);
                }
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
        if let Some(editor) = snapshot.editor.take()
            && self
                .pending_editor
                .take()
                .is_some_and(|(id, library)| id == editor.track_id && library == editor.library_id)
            && snapshot.selected.as_deref() == Some(editor.library_id.as_str())
        {
            self.editing = Some(editor);
        }
        if saving_editor
            || self.editing.as_ref().is_some_and(|editor| {
                snapshot.selected.as_deref() != Some(editor.library_id.as_str())
                    || !snapshot
                        .tracks
                        .iter()
                        .any(|track| track.id == editor.track_id)
            })
        {
            self.editing = None;
        }
        if let Some(addition) = adding_track
            && self
                .playlist_picker
                .as_ref()
                .is_some_and(|draft| draft.track_id == addition.track)
            && let Some(playlist) = snapshot
                .playlists
                .iter()
                .find(|item| item.id == addition.playlist)
        {
            self.playlist_picker = None;
            self.last_playlist_choice = None;
            self.playlist_message = Some(format!("Added {} to {}.", addition.title, playlist.name));
        }
        if let Some(draft) = self.playlist_picker.as_mut() {
            if snapshot.selected.as_ref() != Some(&draft.library_id)
                || !snapshot
                    .tracks
                    .iter()
                    .any(|track| track.id == draft.track_id)
            {
                self.playlist_picker = None;
            } else if !snapshot
                .playlists
                .iter()
                .any(|item| Some(&item.id) == draft.playlist.as_ref())
            {
                draft.playlist = snapshot.playlists.first().map(|item| item.id.clone());
            }
        }
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
    fn it_rolls_back_a_track_add_when_the_resulting_snapshot_cannot_be_read() {
        let root = tempfile::tempdir().unwrap();
        let paths = CataloguePaths::in_directory(root.path());
        let catalogue = paths.open().unwrap();
        let connection = rusqlite::Connection::open(&paths.current).unwrap();
        connection.execute_batch(
            "INSERT INTO libraries (id, name, kind, location) VALUES ('source', 'Test', 'filesystem', '/songs');
             INSERT INTO tracks (id, library_id, title, artist, duration_seconds, file_path, file_size, file_mtime)
             VALUES ('song', 'source', 'Song', '', 60, '/songs/song.mp3', 0, 0),
                    ('broken', 'source', 'Broken', '', 'invalid', '/songs/broken.mp3', 0, 0);",
        ).unwrap();
        let playlist = catalogue.create_playlist("Evening").unwrap();
        let add = Operation::AddToPlaylist(playlist.id.clone(), "song".into());

        assert!(run(&paths, add.clone()).is_err());
        assert!(catalogue.playlist_entries(&playlist.id).unwrap().is_empty());

        connection
            .execute("UPDATE tracks SET duration_seconds=1 WHERE id='broken'", [])
            .unwrap();
        let snapshot = run(&paths, add).unwrap();
        assert_eq!(snapshot.playlist_entries.len(), 1);
    }

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
    fn it_preserves_whitespace_at_the_end_of_the_chosen_folder_location() {
        let root = tempfile::tempdir().unwrap();
        let mut desktop = crate::desktop::Desktop::with_session(Err("test output".into())).0;
        desktop.sources = Sources::new(Some(CataloguePaths::in_directory(root.path())));
        let folder = root.path().join("Live ");
        let _ = desktop.update(Message::AddFolder);
        let _ = desktop.update(Message::FolderPicked(Some(folder.clone())));
        desktop.sources.pending.push_back(Operation::Reload);
        let _ = desktop.update(Message::SubmitFolder);

        let Some(Operation::AddNamedFolder(_, submitted)) = desktop.sources.pending.back() else {
            panic!("the folder was not submitted");
        };
        assert_eq!(submitted, &folder);
    }

    #[test]
    fn it_rolls_back_source_creation_when_a_selected_playlist_cannot_be_read() {
        let root = tempfile::tempdir().unwrap();
        let folder = root.path().join("New source");
        std::fs::create_dir(&folder).unwrap();
        let paths = CataloguePaths::in_directory(root.path());
        let catalogue = paths.open().unwrap();
        let playlist = catalogue.create_playlist("Existing playlist").unwrap();
        let connection = rusqlite::Connection::open(&paths.current).unwrap();
        connection.execute_batch(
            "INSERT INTO libraries (id, name, kind, location) VALUES ('existing', 'Existing', 'filesystem', '/test');
             INSERT INTO tracks (id, library_id, title, artist, duration_seconds, file_path, file_size, file_mtime) VALUES ('bad', 'existing', 'Test', '', 'invalid', '/test/bad.mp3', 0, 0);",
        ).unwrap();
        connection.execute("INSERT INTO playlist_entries (id, playlist_id, track_id, position) VALUES ('entry', ?1, 'bad', 0)", [&playlist.id]).unwrap();

        let add_source = || {
            run_with_selection(
                &paths,
                Operation::AddNamedFolder("For later".into(), folder.clone()),
                None,
                Some(playlist.id.clone()),
            )
        };
        assert!(add_source().is_err());
        assert_eq!(catalogue.libraries().unwrap().len(), 1);
        connection
            .execute("UPDATE tracks SET duration_seconds=1", [])
            .unwrap();
        let snapshot = add_source().unwrap();
        let source = snapshot.libraries.last().unwrap();
        assert_eq!(source.name, "For later");
        assert_eq!(snapshot.selected.as_ref(), Some(&source.id));
        assert_eq!(snapshot.selected_playlist.as_ref(), Some(&playlist.id));
    }

    #[test]
    fn it_keeps_the_source_when_removal_cannot_return_the_updated_catalogue() {
        let root = tempfile::tempdir().unwrap();
        let paths = CataloguePaths::in_directory(root.path());
        let catalogue = paths.open().unwrap();
        let playlist = catalogue.create_playlist("Existing playlist").unwrap();
        let connection = rusqlite::Connection::open(&paths.current).unwrap();
        connection
            .execute_batch(
                "INSERT INTO libraries (id, name, kind, location) VALUES
                     ('victim', 'Source to remove', 'filesystem', '/music/temporary'),
                     ('other', 'Other source', 'filesystem', '/music/other');
                 INSERT INTO tracks (id, library_id, title, artist, duration_seconds, file_path, file_size, file_mtime)
                 VALUES ('bad', 'other', 'Other track', '', 'invalid', '/music/other/track.mp3', 0, 0);",
            )
            .unwrap();
        connection
            .execute(
                "INSERT INTO playlist_entries (id, playlist_id, track_id, position)
                 VALUES ('entry', ?1, 'bad', 0)",
                [&playlist.id],
            )
            .unwrap();

        let remove = || {
            run_with_selection(
                &paths,
                Operation::Remove("victim".into()),
                Some("victim".into()),
                Some(playlist.id.clone()),
            )
        };

        assert!(remove().is_err());
        assert!(
            catalogue
                .libraries()
                .unwrap()
                .iter()
                .any(|source| source.id == "victim"),
            "an unsuccessful catalogue update must leave the source intact"
        );

        connection
            .execute(
                "UPDATE tracks SET duration_seconds = 1 WHERE id = 'bad'",
                [],
            )
            .unwrap();
        let updated = remove().unwrap();
        assert!(updated.libraries.iter().all(|source| source.id != "victim"));
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
    fn it_refreshes_another_source_without_switching_the_selected_source() {
        let directory = tempfile::tempdir().unwrap();
        let paths = CataloguePaths::in_directory(directory.path());
        let mut catalogue = paths.open().unwrap();
        let selected = directory.path().join("selected");
        let refreshed = directory.path().join("refreshed");
        std::fs::create_dir(&selected).unwrap();
        std::fs::create_dir(&refreshed).unwrap();
        let selected = catalogue.add_folder(&selected).unwrap().id;
        let target = catalogue.add_folder(&refreshed).unwrap().id;
        std::fs::copy(
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/tone.mp3"),
            refreshed.join("tone.mp3"),
        )
        .unwrap();

        let snapshot = run_with_selection(
            &paths,
            Operation::Refresh(target.clone()),
            Some(selected.clone()),
            None,
        )
        .unwrap();

        assert_eq!(snapshot.track_counts.get(&target), Some(&1));
        assert_eq!(snapshot.selected, Some(selected));
    }

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

#[cfg(test)]
mod source_selection_tests {
    use super::*;

    struct Libraries {
        _directory: tempfile::TempDir,
        paths: CataloguePaths,
        first: Library,
        second: Library,
    }

    impl Libraries {
        fn new() -> Self {
            let directory = tempfile::tempdir().unwrap();
            let paths = CataloguePaths::in_directory(directory.path());
            let mut catalogue = paths.open().unwrap();
            let first = directory.path().join("Canal Sessions");
            let second = directory.path().join("Harbour Rehearsals");
            std::fs::create_dir(&first).unwrap();
            std::fs::create_dir(&second).unwrap();
            let first = catalogue.add_folder(&first).unwrap();
            let second = catalogue.add_folder(&second).unwrap();
            Self {
                _directory: directory,
                paths,
                first,
                second,
            }
        }

        fn apply(&self, operation: Operation, selected: Option<String>) -> Snapshot {
            run_with_selection(&self.paths, operation, selected, None).unwrap()
        }
    }

    #[test]
    fn it_waits_for_the_user_to_choose_a_source_after_startup() {
        let libraries = Libraries::new();
        let startup = libraries.apply(Operation::Startup, None);
        assert_eq!(startup.libraries.len(), 2);
        assert_eq!(startup.selected, None);

        let chosen = libraries.apply(
            Operation::Select(libraries.second.id.clone()),
            startup.selected,
        );
        assert_eq!(chosen.selected.as_ref(), Some(&libraries.second.id));
    }

    #[test]
    fn it_keeps_the_library_unselected_after_removing_the_active_source_and_reloading() {
        let libraries = Libraries::new();
        let snapshot = libraries.apply(
            Operation::Remove(libraries.first.id.clone()),
            Some(libraries.first.id.clone()),
        );
        assert_eq!(
            snapshot.libraries.as_slice(),
            std::slice::from_ref(&libraries.second)
        );
        assert_eq!(snapshot.selected, None);

        let refreshed = libraries.apply(Operation::Reload, snapshot.selected);
        assert_eq!(refreshed.selected, None);
        assert!(refreshed.tracks.is_empty());
    }

    #[test]
    fn it_keeps_the_selected_source_when_a_different_source_is_removed() {
        let libraries = Libraries::new();
        let snapshot = libraries.apply(
            Operation::Remove(libraries.first.id.clone()),
            Some(libraries.second.id.clone()),
        );
        assert_eq!(snapshot.selected.as_ref(), Some(&libraries.second.id));
    }
}
