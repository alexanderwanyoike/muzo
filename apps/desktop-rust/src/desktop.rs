mod dropbox_panel;
pub(crate) mod library_sources;
mod mini_focus;
mod sidebar;
pub use mini_focus::Control as MiniControl;
mod updates;
mod view;
mod windows;

use std::{path::PathBuf, time::Duration};

use iced::{Element, Subscription, Task, Theme, widget::image, window};
use rodio::{DeviceSinkBuilder, MixerDeviceSink};

use crate::{
    local_track::read_track,
    playback::Playback,
    session::{Session, SessionError},
    track::Track,
};

use library_sources::{CataloguePaths, Operation, Snapshot, Sources};
use windows::Windows;

#[derive(Debug, Clone)]
pub enum Message {
    OpenFiles,
    FilesPicked(Vec<PathBuf>),
    AddFolder,
    FolderPicked(Option<PathBuf>),
    SourceSearch(String),
    SourceOperation(Operation),
    SourcesUpdated(Result<Snapshot, String>),
    SourceWatchTick,
    Dropbox(dropbox_panel::Action),
    DropboxReady(Result<dropbox_panel::Snapshot, String>),
    Material(String),
    AppearanceMode(String),
    Accent(String),
    CheckUpdate,
    AppImageIntegration(Result<(), String>),
    UpdateReceived(updates::Status),
    DownloadUpdate(crate::distribution::Update),
    InstallUpdate(crate::distribution::Update, PathBuf),
    UpdateInstalled(Result<(), String>),
    RequestSourceRemoval(String),
    CancelSourceRemoval,
    PlayIndexed(usize),
    PlayPlaylist(usize),
    PlaylistName(String),
    CreatePlaylist,
    EditTrack(crate::catalogue::IndexedTrack),
    EditField(usize, String),
    SaveTrackEdit(bool),
    CloseTrackEdit,
    RemoteTrackReady(u64, Result<Track, String>),
    FilesRead(Result<Vec<Track>, String>),
    Select(usize),
    TogglePlayback,
    RetryTrack,
    Previous,
    Next,
    Seek(f32),
    SeekRelative(i8),
    SetVolume(f32),
    SetMiniVolume(f32),
    ResizeSidebar(f32),
    SaveSidebar(u64, crate::appearance::sidebar::SidebarWidth),
    Tick,
    ShowMini,
    ToggleMini(window::Id),
    MiniOpened(window::Id),
    LibraryVisibilityChecked(window::Id, bool),
    ShowLibrary,
    ToggleQueue,
    CycleMiniFocus(window::Id, bool),
    ActivateMiniFocus(window::Id),
    MiniSpace(window::Id),
    MiniArrow(window::Id, i8),
    ClickMiniControl(MiniControl),
    CloseRequested(window::Id),
    MiniResized(window::Id, iced::Size),
    CheckQueueSize(window::Id, u64),
    QueueResizeTimedOut(window::Id, u64, iced::Size),
}

pub struct Desktop {
    windows: Windows,
    session: Option<Session>,
    output: Option<MixerDeviceSink>,
    error: Option<String>,
    artwork: Option<image::Handle>,
    sources: Sources,
    dropbox: dropbox_panel::Settings,
    playback_request: u64,
    pending_track: Option<usize>,
    failed_track: Option<usize>,
    mini_focus: Option<mini_focus::Control>,
    sidebar: sidebar::Sidebar,
    sidebar_revision: u64,
    appearance: crate::appearance::Appearance,
    update_status: updates::Status,
}

impl Desktop {
    pub fn boot() -> (Self, Task<Message>) {
        let output =
            DeviceSinkBuilder::open_default_sink().map_err(crate::playback::PlaybackError::from);
        let session = output
            .as_ref()
            .map(|device| Session::new(Playback::new(device.mixer().clone())))
            .map_err(ToString::to_string);
        let (mut desktop, open) = Self::with_session(session);
        desktop.output = output.ok();
        desktop.sources = Sources::new(CataloguePaths::from_environment());
        if let Some(path) = desktop.sources.paths().map(|paths| paths.appearance()) {
            let appearance = if std::env::var_os("MUZO_DATA_DIR").is_none() {
                CataloguePaths::legacy_appearance().map_or_else(
                    || crate::appearance::load(&path),
                    |legacy| crate::appearance::load_or_import(&path, &legacy),
                )
            } else {
                crate::appearance::load(&path)
            };
            match appearance {
                Ok(appearance) => desktop.appearance = appearance,
                Err(error) => desktop.error = Some(error.to_string()),
            }
        }
        if let Some(path) = desktop.sources.paths().map(|paths| paths.sidebar()) {
            let legacy = std::env::var_os("MUZO_DATA_DIR")
                .is_none()
                .then(CataloguePaths::legacy_appearance)
                .flatten();
            match crate::appearance::sidebar::load_or_import(&path, legacy.as_deref()) {
                Ok(width) => desktop.sidebar.set(width),
                Err(error) => desktop.error = Some(error.to_string()),
            }
        }
        let load_sources = desktop.sources.start(Operation::Startup);
        let files = std::env::args_os().skip(1).map(PathBuf::from).collect();
        (
            desktop,
            open.chain(Task::batch([
                load_sources,
                auto_update(),
                appimage_integration(),
                Task::done(Message::FilesPicked(files)),
            ])),
        )
    }

    pub fn with_session(session: Result<Session, String>) -> (Self, Task<Message>) {
        let (windows, open) = Windows::open();
        let (session, error) = match session {
            Ok(session) => (Some(session), None),
            Err(error) => (None, Some(error)),
        };
        (
            Self {
                windows,
                session,
                output: None,
                error,
                artwork: None,
                sources: Sources::new(None),
                dropbox: dropbox_panel::Settings::default(),
                playback_request: 0,
                pending_track: None,
                failed_track: None,
                mini_focus: None,
                sidebar: sidebar::Sidebar::default(),
                sidebar_revision: 0,
                appearance: crate::appearance::Appearance::default(),
                update_status: updates::Status::Idle,
            },
            open,
        )
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::AddFolder => Task::perform(
                async {
                    rfd::AsyncFileDialog::new()
                        .pick_folder()
                        .await
                        .map(|folder| folder.path().to_owned())
                },
                Message::FolderPicked,
            ),
            Message::FolderPicked(Some(path)) => self.sources.start(Operation::Add(path)),
            Message::FolderPicked(None) => Task::none(),
            Message::SourceSearch(query) => {
                self.sources.search = query;
                Task::none()
            }
            Message::SourceOperation(operation) => {
                self.sources.pending_remove = None;
                self.sources.start(operation)
            }
            Message::SourcesUpdated(result) => {
                if let (Some(previous), Ok(next)) = (&self.sources.snapshot, &result) {
                    let removed: Vec<_> = previous
                        .libraries
                        .iter()
                        .filter(|old| !next.libraries.iter().any(|source| old.id == source.id))
                        .map(|source| source.id.clone())
                        .collect();
                    for library in removed {
                        self.remove_queued_library(&library);
                    }
                }
                self.sources.apply(result);
                Task::none()
            }
            Message::SourceWatchTick => {
                let local = self.sources.poll();
                if self.dropbox.due() {
                    let remote = self
                        .dropbox
                        .update(dropbox_panel::Action::Sync, self.sources.paths());
                    Task::batch([local, remote])
                } else {
                    local
                }
            }
            Message::Dropbox(action) => self.dropbox.update(action, self.sources.paths()),
            Message::AppImageIntegration(result) => {
                if let Err(error) = result {
                    self.error = Some(error);
                }
                Task::none()
            }
            Message::CheckUpdate => {
                if matches!(self.update_status, updates::Status::Installing) {
                    return Task::none();
                }
                self.update_status = updates::Status::Checking;
                updates::check()
            }
            Message::UpdateReceived(status) => {
                if !matches!(self.update_status, updates::Status::Installing) {
                    self.update_status = status;
                }
                Task::none()
            }
            Message::DownloadUpdate(update) => {
                if !matches!(self.update_status, updates::Status::Available(_)) {
                    return Task::none();
                }
                if cfg!(target_os = "macos")
                    || (cfg!(target_os = "linux") && std::env::var_os("APPIMAGE").is_none())
                {
                    if let Err(error) = open::that_detached(update.release_page()) {
                        self.update_status = updates::Status::Error(error.to_string());
                    }
                    return Task::none();
                }
                if let Some(paths) = self.sources.paths() {
                    self.update_status = updates::Status::Downloading(0);
                    return updates::download(update, paths.update_cache());
                }
                Task::none()
            }
            Message::InstallUpdate(update, path) => {
                if !matches!(self.update_status, updates::Status::Ready(_, _)) {
                    return Task::none();
                }
                self.update_status = updates::Status::Installing;
                Task::perform(
                    async move {
                        tokio::task::spawn_blocking(move || {
                            updates::launch(&update, &path).map_err(|error| error.to_string())
                        })
                        .await
                        .map_err(|error| format!("Could not install the update: {error}"))?
                    },
                    Message::UpdateInstalled,
                )
            }
            Message::UpdateInstalled(result) => match result {
                Ok(()) => iced::exit(),
                Err(error) => {
                    self.update_status = updates::Status::Error(error.to_string());
                    Task::none()
                }
            },
            Message::Material(material) => {
                self.appearance.theme = material;
                self.save_appearance()
            }
            Message::AppearanceMode(mode) => {
                self.appearance.mode = mode;
                self.save_appearance()
            }
            Message::Accent(accent) => {
                self.appearance.accent = accent;
                self.save_appearance()
            }
            Message::DropboxReady(mut result) => {
                if let Ok(snapshot) = &mut result {
                    for library in std::mem::take(&mut snapshot.disconnected_libraries) {
                        self.remove_queued_library(&library);
                    }
                }
                self.dropbox.apply(result);
                self.sources.reload()
            }
            Message::RequestSourceRemoval(id) => {
                self.sources.pending_remove = Some(id);
                Task::none()
            }
            Message::CancelSourceRemoval => {
                self.sources.pending_remove = None;
                Task::none()
            }
            Message::PlayIndexed(index) => self.play_indexed(index, false),
            Message::PlayPlaylist(index) => self.play_indexed(index, true),
            Message::PlaylistName(name) => {
                self.sources.playlist_name = name;
                Task::none()
            }
            Message::CreatePlaylist => {
                let name = std::mem::take(&mut self.sources.playlist_name);
                self.sources.start(Operation::CreatePlaylist(name))
            }
            Message::EditTrack(track) => self
                .sources
                .start(Operation::LoadTrackEditor(track.id, track.library_id)),
            Message::EditField(index, value) => {
                if let Some((_, _, fields)) = &mut self.sources.editing
                    && let Some(field) = fields.get_mut(index)
                {
                    *field = value;
                }
                Task::none()
            }
            Message::SaveTrackEdit(clear) => self.save_track_edit(clear),
            Message::CloseTrackEdit => {
                self.sources.editing = None;
                Task::none()
            }
            Message::RemoteTrackReady(request, result) => {
                if request != self.playback_request {
                    return Task::none();
                }
                let Some(index) = self.pending_track.take() else {
                    return Task::none();
                };
                match result {
                    Ok(track) => self.transport(|session| {
                        session.replace_track(index, track)?;
                        session.select(index)
                    }),
                    Err(error) => {
                        self.failed_track = Some(index);
                        self.error = Some(error);
                        Task::none()
                    }
                }
            }
            Message::OpenFiles => Task::perform(
                async {
                    rfd::AsyncFileDialog::new()
                        .add_filter("Audio", &crate::catalogue::MUSIC_EXTENSIONS)
                        .pick_files()
                        .await
                        .unwrap_or_default()
                        .into_iter()
                        .map(|file| file.path().to_owned())
                        .collect()
                },
                Message::FilesPicked,
            ),
            Message::FilesPicked(paths) if paths.is_empty() => Task::none(),
            Message::FilesPicked(paths) => Task::perform(
                async move {
                    tokio::task::spawn_blocking(move || paths.into_iter().map(read_track).collect())
                        .await
                        .map_err(|error| format!("Could not read the selected tracks: {error}"))
                },
                Message::FilesRead,
            ),
            Message::FilesRead(Ok(tracks)) => {
                if let Some(session) = &mut self.session
                    && !tracks.is_empty()
                {
                    let first = session.tracks().len();
                    session.extend(tracks);
                    if session.current().is_none() {
                        return self.transport(|session| session.select(first));
                    }
                }
                Task::none()
            }
            Message::FilesRead(Err(error)) => {
                self.error = Some(error);
                Task::none()
            }
            Message::Select(index) => self.select_track(index),
            Message::TogglePlayback => {
                if let Some(index) = self.failed_track
                    && self.session.as_ref().and_then(Session::current).is_none()
                {
                    self.select_track(index)
                } else {
                    self.transport(Session::toggle)
                }
            }
            Message::RetryTrack => self
                .failed_track
                .map_or_else(Task::none, |index| self.select_track(index)),
            Message::Previous => self.move_track(-1),
            Message::Next => self.move_track(1),
            Message::Seek(seconds) => {
                self.transport(|session| session.seek(Duration::from_secs_f32(seconds)))
            }
            Message::SeekRelative(seconds) => self.transport(|session| {
                let (position, _) = session.observe();
                let target = if seconds >= 0 {
                    position + Duration::from_secs(seconds as u64)
                } else {
                    position.saturating_sub(Duration::from_secs(seconds.unsigned_abs().into()))
                };
                session.seek(target)
            }),
            Message::ResizeSidebar(ratio) => {
                let width = self.sidebar.drag(ratio);
                self.sidebar_revision += 1;
                let revision = self.sidebar_revision;
                Task::perform(
                    async { tokio::time::sleep(Duration::from_millis(180)).await },
                    move |()| Message::SaveSidebar(revision, width),
                )
            }
            Message::SaveSidebar(revision, width) => {
                if revision != self.sidebar_revision {
                    return Task::none();
                }
                if let Some(path) = self.sources.paths().map(|paths| paths.sidebar())
                    && let Err(error) = crate::appearance::sidebar::save(&path, width)
                {
                    self.error = Some(error.to_string());
                }
                Task::none()
            }
            Message::SetVolume(value) => self.transport(|session| {
                session.set_volume(value);
                Ok(())
            }),
            Message::SetMiniVolume(value) => {
                self.mini_focus = Some(MiniControl::Volume);
                self.update(Message::SetVolume(value))
            }
            Message::Tick => {
                if self.pending_track.is_some() || self.failed_track.is_some() {
                    return Task::none();
                }
                if self.session.as_ref().is_some_and(Session::finished) {
                    let next = self.session.as_ref().and_then(|session| {
                        let index = session.current_index()? + 1;
                        (index < session.tracks().len()).then_some(index)
                    });
                    if let Some(index) = next {
                        return self.select_track(index);
                    }
                }
                self.refresh_transport(Session::advance_if_finished, false);
                Task::none()
            }
            Message::ToggleMini(id) => {
                if self.windows.is_mini(id) {
                    self.windows.show_library()
                } else {
                    self.mini_focus = None;
                    self.windows.show_mini()
                }
            }
            Message::ShowMini => {
                self.mini_focus = None;
                self.windows.show_mini()
            }
            Message::CycleMiniFocus(id, backwards) => {
                if self.windows.is_mini(id) {
                    let queue_len = self
                        .session
                        .as_ref()
                        .filter(|_| self.windows.queue_open())
                        .map_or(0, |session| session.tracks().len());
                    self.mini_focus =
                        MiniControl::next(self.mini_focus, backwards, queue_len, |control| {
                            control.enabled(
                                self.session.as_ref(),
                                self.failed_track.is_some(),
                                self.windows.queue_pending(),
                                self.windows.queue_open(),
                            )
                        });
                    if let Some(MiniControl::Track(index)) = self.mini_focus {
                        return mini_focus::reveal_track(index);
                    }
                }
                Task::none()
            }
            Message::ActivateMiniFocus(id) => {
                if let Some(control) = self.mini_focus.filter(|_| self.windows.is_mini(id)) {
                    return self.update(Message::ClickMiniControl(control));
                }
                Task::none()
            }
            Message::ClickMiniControl(control) => {
                if let Some(message) = control.message(
                    self.session.as_ref(),
                    self.failed_track.is_some(),
                    self.windows.queue_pending(),
                    self.windows.queue_open(),
                ) {
                    self.mini_focus = Some(control);
                    return self.update(message);
                }
                Task::none()
            }
            Message::MiniSpace(id) => {
                if self.windows.is_mini(id) && self.mini_focus.is_some() {
                    self.update(Message::ActivateMiniFocus(id))
                } else {
                    self.update(Message::TogglePlayback)
                }
            }
            Message::MiniArrow(id, step) => {
                if self.windows.is_mini(id) && self.mini_focus == Some(MiniControl::Volume) {
                    let volume = self.session.as_ref().map_or(1.0, Session::volume);
                    self.update(Message::SetVolume(
                        (volume + 0.05 * f32::from(step)).clamp(0.0, 1.0),
                    ))
                } else {
                    self.update(Message::SeekRelative(step * 5))
                }
            }
            Message::MiniOpened(id) => self.windows.mini_opened(id),
            Message::LibraryVisibilityChecked(id, wayland) => {
                self.windows.library_visibility_checked(id, wayland)
            }
            Message::ShowLibrary => self.windows.show_library(),
            Message::ToggleQueue => {
                self.error = None;
                if self.windows.queue_open()
                    && matches!(self.mini_focus, Some(MiniControl::Track(_)))
                {
                    self.mini_focus = Some(MiniControl::Queue);
                }
                self.windows.toggle_queue()
            }
            Message::CheckQueueSize(id, generation) => {
                window::size(id).map(move |size| Message::QueueResizeTimedOut(id, generation, size))
            }
            Message::QueueResizeTimedOut(id, generation, size) => {
                let (task, result) = self.windows.resize_timed_out(id, generation, size);
                if let Err(error) = result {
                    self.error = Some(error.to_string());
                }
                task
            }

            Message::CloseRequested(id) => self.windows.close_requested(id),
            Message::MiniResized(id, size) => {
                if self.windows.is_mini(id) {
                    self.windows.resized(id, size);
                } else if self.windows.is_library(id) {
                    self.sidebar.layout_width(size.width);
                }
                Task::none()
            }
        }
    }

    fn save_appearance(&mut self) -> Task<Message> {
        if let Some(path) = self.sources.paths().map(|paths| paths.appearance())
            && let Err(error) = crate::appearance::save(&path, &self.appearance)
        {
            self.error = Some(error.to_string());
        }
        Task::none()
    }

    fn save_track_edit(&mut self, clear: bool) -> Task<Message> {
        let Some((id, library, fields)) = self.sources.editing.as_ref() else {
            return Task::none();
        };
        let non_empty = |index: usize| {
            let value = fields[index].trim();
            (!value.is_empty()).then(|| value.to_owned())
        };
        let mut edit = crate::catalogue::MetadataOverride::default();
        if !clear {
            let values: Result<Vec<_>, _> = [3, 4, 6]
                .into_iter()
                .map(|index| {
                    non_empty(index)
                        .map(|value| value.parse::<u32>())
                        .transpose()
                })
                .collect();
            let Ok(values) = values else {
                self.sources.error = Some("Track, disc and year must be whole numbers.".into());
                return Task::none();
            };
            edit = crate::catalogue::MetadataOverride {
                title: non_empty(0),
                artist: non_empty(1),
                album: non_empty(2),
                track_number: values[0],
                disc_number: values[1],
                genre: non_empty(5),
                year: values[2],
            };
        }
        if self.sources.busy() {
            return Task::none();
        }
        self.sources.saving_editor = true;
        self.sources
            .start(Operation::EditTrack(id.clone(), library.clone(), edit))
    }

    fn play_indexed(&mut self, index: usize, playlist: bool) -> Task<Message> {
        let Some(snapshot) = &self.sources.snapshot else {
            return Task::none();
        };
        let tracks = if playlist {
            snapshot
                .playlist_entries
                .iter()
                .filter_map(|entry| entry.track.clone())
                .collect()
        } else {
            snapshot.tracks.clone()
        };
        if index >= tracks.len() || self.session.is_none() {
            return Task::none();
        }
        let remote_libraries: std::collections::HashSet<_> = snapshot
            .libraries
            .iter()
            .filter(|library| library.kind == "dropbox")
            .map(|library| library.id.clone())
            .collect();
        let tracks = tracks
            .into_iter()
            .map(|track| {
                let is_remote = remote_libraries.contains(&track.library_id);
                let path = track.path.to_string_lossy().into_owned();
                let item = Track::new(
                    if is_remote {
                        PathBuf::new()
                    } else {
                        track.path
                    },
                    track.title,
                    Some(track.artist),
                    track.album,
                    None,
                )
                .with_indexed_id(track.id)
                .with_library(track.library_id.clone());
                if is_remote {
                    item.with_remote(track.library_id, path)
                } else {
                    item
                }
            })
            .collect();
        if let Some(session) = &mut self.session {
            session.replace(tracks);
        }
        if matches!(self.mini_focus, Some(MiniControl::Track(_))) {
            self.mini_focus = Some(MiniControl::Queue);
        }
        self.artwork = None;
        self.select_track(index)
    }

    fn remove_queued_library(&mut self, library: &str) {
        let Some(session) = self.session.as_mut() else {
            return;
        };
        if let Some(MiniControl::Track(index)) = self.mini_focus {
            self.mini_focus = Some(
                session
                    .index_without_library(index, library)
                    .map_or(MiniControl::Queue, MiniControl::Track),
            );
        }
        self.pending_track = self
            .pending_track
            .and_then(|index| session.index_without_library(index, library));
        if let Some(index) = self.failed_track {
            self.failed_track = session.index_without_library(index, library);
            if self.failed_track.is_none() {
                self.error = None;
            }
        }
        session.remove_library(library);
        if session.current().is_none() {
            self.artwork = None;
        }
    }

    fn move_track(&mut self, step: isize) -> Task<Message> {
        let index = self.session.as_ref().and_then(|session| {
            let index = session.current_index()?.checked_add_signed(step)?;
            (index < session.tracks().len()).then_some(index)
        });
        index.map_or_else(Task::none, |index| self.select_track(index))
    }

    fn select_track(&mut self, index: usize) -> Task<Message> {
        self.playback_request += 1;
        let request = self.playback_request;
        let Some(track) = self
            .session
            .as_ref()
            .and_then(|session| session.tracks().get(index))
            .cloned()
        else {
            return Task::none();
        };
        self.pending_track = None;
        self.failed_track = None;
        if let Some((library, path)) = track.remote().filter(|_| !track.path().is_file()) {
            let Some(paths) = self.sources.paths() else {
                return Task::none();
            };
            let library = library.to_owned();
            let path = path.to_owned();
            self.pending_track = Some(index);
            return Task::perform(
                async move {
                    tokio::task::spawn_blocking(move || {
                        let local = dropbox_panel::remote_audio(&paths, &library, &path)
                            .map_err(|error| error.to_string())?;
                        Ok(prepare_dropbox_track(&paths, track, local))
                    })
                    .await
                    .map_err(|error| format!("Could not prepare the Dropbox track: {error}"))?
                },
                move |result| Message::RemoteTrackReady(request, result),
            );
        }
        if track.indexed_id().is_some() && track.artwork().is_none() && track.remote().is_none() {
            self.pending_track = Some(index);
            return Task::perform(
                async move {
                    tokio::task::spawn_blocking(move || {
                        let path = track.path().to_owned();
                        let metadata = read_track(path.clone());
                        track.with_audio(path, &metadata)
                    })
                    .await
                    .map_err(|error| format!("Could not read track artwork: {error}"))
                },
                move |result| Message::RemoteTrackReady(request, result),
            );
        }
        self.transport(|session| session.select(index))
    }

    fn transport(
        &mut self,
        operation: impl FnOnce(&mut Session) -> Result<(), SessionError>,
    ) -> Task<Message> {
        self.refresh_transport(operation, true);
        Task::none()
    }

    fn refresh_transport(
        &mut self,
        operation: impl FnOnce(&mut Session) -> Result<(), SessionError>,
        clear_on_success: bool,
    ) {
        if let Some(session) = &mut self.session {
            let current = session.current_index();
            let before = session.start_generation();
            match operation(session) {
                Ok(()) => {
                    if let Some(id) = session.current().and_then(Track::indexed_id)
                        && session.is_playing()
                        && before != session.start_generation()
                    {
                        self.sources.queued_plays.push(id.to_owned());
                    }
                    if clear_on_success || current != session.current_index() {
                        self.error = None;
                    }
                }
                Err(error) => self.error = Some(error.to_string()),
            }
            if current != session.current_index() {
                self.artwork = session
                    .current()
                    .and_then(Track::artwork)
                    .map(|encoded| image::Handle::from_bytes(encoded.to_vec()));
            }
        }
    }

    pub fn subscription(&self) -> Subscription<Message> {
        Subscription::batch([
            window::close_requests().map(Message::CloseRequested),
            iced::time::every(Duration::from_secs(1)).map(|_| Message::SourceWatchTick),
            window::resize_events().map(|(id, size)| Message::MiniResized(id, size)),
            if self.session.as_ref().is_some_and(Session::is_playing) {
                iced::time::every(Duration::from_millis(100)).map(|_| Message::Tick)
            } else {
                Subscription::none()
            },
            iced::event::listen_with(|event, status, id| {
                if status == iced::event::Status::Captured {
                    return None;
                }
                let iced::Event::Keyboard(iced::keyboard::Event::KeyPressed {
                    key, modifiers, ..
                }) = event
                else {
                    return None;
                };
                use iced::keyboard::{Key, key::Named};
                match (key.as_ref(), modifiers.command()) {
                    (Key::Named(Named::Tab), false) => {
                        Some(Message::CycleMiniFocus(id, modifiers.shift()))
                    }
                    (Key::Named(Named::Enter), false) => Some(Message::ActivateMiniFocus(id)),
                    (Key::Named(Named::Space), false) => Some(Message::MiniSpace(id)),
                    (Key::Named(Named::Escape), false) => Some(Message::ShowLibrary),
                    (Key::Named(Named::ArrowLeft), false) => Some(Message::MiniArrow(id, -1)),
                    (Key::Named(Named::ArrowRight), false) => Some(Message::MiniArrow(id, 1)),
                    (Key::Character("m"), true) => Some(Message::ToggleMini(id)),
                    (Key::Character("o"), true) => Some(Message::OpenFiles),
                    (Key::Character("q"), true) if !cfg!(target_os = "macos") => {
                        Some(Message::ToggleQueue)
                    }
                    (Key::Character("q"), true) if modifiers.shift() => Some(Message::ToggleQueue),
                    _ => None,
                }
            }),
        ])
    }

    pub fn title(&self, id: window::Id) -> String {
        if self.windows.is_mini(id) {
            "Muzo · Mini player"
        } else {
            "Muzo"
        }
        .into()
    }

    pub fn view(&self, id: window::Id) -> Element<'_, Message> {
        if self.windows.is_mini(id) {
            view::mini(
                self.session.as_ref(),
                (self.windows.queue_open(), self.windows.queue_pending()),
                self.artwork.as_ref(),
                (self.error.as_deref(), self.failed_track.is_some()),
                self.mini_focus,
            )
        } else {
            view::library(
                &self.sources,
                &self.dropbox,
                (&self.appearance, &self.update_status, &self.sidebar),
                self.session.as_ref(),
                self.artwork.as_ref(),
                (self.error.as_deref(), self.failed_track.is_some()),
            )
        }
    }

    pub fn theme(&self, _id: window::Id) -> Theme {
        self.appearance.theme()
    }

    #[cfg(test)]
    pub fn now_playing(&self) -> Option<(usize, Duration)> {
        let session = self.session.as_ref()?;
        Some((session.current_index()?, session.observe().0))
    }
}

fn auto_update() -> Task<Message> {
    if (cfg!(target_os = "linux") && std::env::var_os("APPIMAGE").is_some())
        || (cfg!(any(target_os = "windows", target_os = "macos")) && !cfg!(debug_assertions))
    {
        Task::done(Message::CheckUpdate)
    } else {
        Task::none()
    }
}

fn appimage_integration() -> Task<Message> {
    #[cfg(target_os = "linux")]
    if let (Some(image), Some(base)) = (std::env::var_os("APPIMAGE"), directories::BaseDirs::new())
    {
        let data = base.data_dir().to_owned();
        return Task::perform(
            async move {
                tokio::task::spawn_blocking(move || {
                    crate::distribution::appimage::register(std::path::Path::new(&image), &data)
                        .map_err(|error| {
                            format!("Could not add Muzo to the application menu: {error}")
                        })
                })
                .await
                .map_err(|error| format!("Could not add Muzo to the application menu: {error}"))?
            },
            Message::AppImageIntegration,
        );
    }
    Task::none()
}

fn prepare_dropbox_track(paths: &CataloguePaths, track: Track, local: PathBuf) -> Track {
    let (metadata, title) = crate::local_track::read_track_with_embedded_title(local.clone());
    if let Some(id) = track.indexed_id()
        && let Err(error) = paths.open().and_then(|catalogue| {
            catalogue.cache_dropbox_metadata(
                id,
                title.as_deref(),
                metadata.artist(),
                metadata.album(),
            )
        })
    {
        eprintln!("Could not cache Dropbox track metadata: {error}");
    }
    track.with_audio(local, &metadata)
}

#[cfg(test)]
#[path = "desktop/queue_tests.rs"]
mod queue_tests;
