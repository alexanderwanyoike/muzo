mod appearance_focus;
mod cover;
mod dropbox_panel;
pub(crate) mod library_sources;
pub(crate) mod material;
mod mini_focus;
mod row_cover;
mod settings_scroll;
mod sidebar;
pub use mini_focus::Control as MiniControl;
mod updates;
mod view;
mod windows;

use std::{path::PathBuf, time::Duration};

use iced::{Element, Subscription, Task, Theme, window};
use rodio::{DeviceSinkBuilder, MixerDeviceSink};

use crate::{
    local_track::read_track,
    playback::Playback,
    session::{Session, SessionError},
    track::Track,
};

use cover::Cover;
use library_sources::{CataloguePaths, Operation, Snapshot, Sources};
use windows::Windows;

#[derive(Debug, Clone)]
pub enum Message {
    OpenFiles,
    FilesPicked(Vec<PathBuf>),
    AddFolder,
    FolderPicked(Option<PathBuf>),
    FolderName(String),
    FolderLocation(String),
    SubmitFolder,
    SourceSearch(String),
    OpenSourceSettings,
    LibraryPage(LibraryPage),
    SourceOperation(Operation),
    SourcesUpdated(Result<Snapshot, String>),
    SourceWatchTick,
    PlayRecorded(String, Result<u32, String>),
    Dropbox(dropbox_panel::Action),
    DropboxReady(Result<dropbox_panel::Snapshot, String>),
    DropboxLibraryRefreshed(String, Result<(String, usize), dropbox_panel::RefreshError>),
    Material(&'static str),
    AppearanceMode(&'static str),
    Accent(&'static str),
    AppearancePointerPressed(window::Id),
    CheckUpdate,
    AppImageIntegration(Result<(), String>),
    UpdateReceived(updates::Status),
    DownloadUpdate(crate::distribution::Update),
    InstallUpdate(crate::distribution::Update, PathBuf),
    ConfirmInstallUpdate,
    CancelInstallUpdate,
    UpdateInstalled(Result<(), String>),
    RequestSourceRemoval(String),
    CancelSourceRemoval,
    PlayIndexed(usize),
    PlayPlaylist(String),
    StartPlaylist(String),
    PlaylistName(String),
    PlaylistSearch(String),
    ShowPlaylistForm(bool),
    CreatePlaylist,
    EditTrack(crate::catalogue::IndexedTrack),
    EditField(usize, String),
    SaveTrackEdit(bool),
    CloseTrackEdit,
    RemoteTrackReady(u64, Result<Track, String>),
    FilesRead(Result<row_cover::OpenedFiles, String>),
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

enum CatalogueSelection<'a> {
    Source(usize),
    PlaylistEntry(&'a str, bool),
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum LibraryPage {
    #[default]
    Songs,
    Playlists,
    Settings,
}

pub struct Desktop {
    windows: Windows,
    session: Option<Session>,
    output: Option<MixerDeviceSink>,
    error: Option<String>,
    cover: Cover,
    direct_covers: row_cover::DirectCovers,
    queue_covers: row_cover::Covers,
    sources: Sources,
    dropbox: dropbox_panel::Settings,
    playback_request: u64,
    pending_track: Option<usize>,
    failed_track: Option<usize>,
    mini_focus: Option<mini_focus::Control>,
    sidebar: sidebar::Sidebar,
    page: LibraryPage,
    appearance_focus: Option<appearance_focus::Choice>,
    sidebar_revision: u64,
    plays_pending: usize,
    close_after_history: Option<window::Id>,
    install_after_history: bool,
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
                cover: Cover::default(),
                direct_covers: Default::default(),
                queue_covers: Default::default(),
                sources: Sources::new(None),
                dropbox: dropbox_panel::Settings::default(),
                playback_request: 0,
                pending_track: None,
                failed_track: None,
                mini_focus: None,
                sidebar: sidebar::Sidebar::default(),
                page: LibraryPage::default(),
                appearance_focus: None,
                sidebar_revision: 0,
                plays_pending: 0,
                close_after_history: None,
                install_after_history: false,
                appearance: crate::appearance::Appearance::default(),
                update_status: updates::Status::Idle,
            },
            open,
        )
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::AddFolder => {
                if !self.sources.folder_editable() {
                    return Task::none();
                }
                self.sources.choosing_folder = true;
                Task::perform(
                    async {
                        rfd::AsyncFileDialog::new()
                            .pick_folder()
                            .await
                            .map(|folder| folder.path().to_owned())
                    },
                    Message::FolderPicked,
                )
            }
            Message::FolderPicked(path) => {
                if !std::mem::take(&mut self.sources.choosing_folder) {
                    return Task::none();
                }
                if let Some(path) = path {
                    let Some(location) = path.to_str() else {
                        self.sources.error =
                            Some(crate::catalogue::CatalogueError::UnsupportedPath.to_string());
                        return Task::none();
                    };
                    if self.sources.folder_name.trim().is_empty() {
                        self.sources.folder_name = path
                            .file_name()
                            .unwrap_or(path.as_os_str())
                            .to_string_lossy()
                            .into_owned();
                    }
                    self.sources.folder_location = location.into();
                }
                Task::none()
            }
            Message::FolderName(name) => {
                if self.sources.folder_editable() {
                    self.sources.folder_name = name;
                }
                Task::none()
            }
            Message::FolderLocation(location) => {
                if self.sources.folder_editable() {
                    self.sources.folder_location = location;
                }
                Task::none()
            }
            Message::SubmitFolder => {
                if self.sources.busy() || !self.sources.folder_editable() {
                    return Task::none();
                }
                if self.sources.folder_location.trim().is_empty() {
                    self.sources.error =
                        Some(crate::catalogue::CatalogueError::EmptyLibraryLocation.to_string());
                    return Task::none();
                }
                self.sources.start(Operation::AddNamedFolder(
                    self.sources.folder_name.clone(),
                    PathBuf::from(&self.sources.folder_location),
                ))
            }
            Message::LibraryPage(page) => {
                self.page = page;
                self.appearance_focus = None;
                self.sources.pending_remove = None;
                self.sources.editing = None;
                Task::none()
            }
            Message::OpenSourceSettings => {
                let _ = self.update(Message::LibraryPage(LibraryPage::Settings));
                settings_scroll::reveal_sources()
            }
            Message::SourceSearch(query) => {
                self.sources.search = query;
                self.sources.pending_remove = None;
                Task::none()
            }
            Message::SourceOperation(operation) => {
                if matches!(operation, Operation::Select(_)) {
                    self.page = LibraryPage::Songs;
                }
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
                        if self
                            .dropbox
                            .refresh_status
                            .as_ref()
                            .is_some_and(|(source, _)| *source == library)
                        {
                            self.dropbox.refresh_status = None;
                        }
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
                if !self.update_status.can_check() || !updates::installed_build() {
                    return Task::none();
                }
                self.update_status = updates::Status::Checking;
                updates::check()
            }
            Message::UpdateReceived(status) => {
                if self.update_status.accepts(&status) {
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
                self.update_status = updates::Status::Confirming(update, path);
                Task::none()
            }
            Message::CancelInstallUpdate => {
                if let updates::Status::Confirming(update, path) = &self.update_status {
                    self.update_status = updates::Status::Ready(update.clone(), path.clone());
                }
                Task::none()
            }
            Message::ConfirmInstallUpdate => {
                let updates::Status::Confirming(update, path) = &self.update_status else {
                    return Task::none();
                };
                let (update, path) = (update.clone(), path.clone());
                if self.plays_pending > 0 {
                    self.install_after_history = true;
                    return Task::none();
                }
                self.install_after_history = false;
                if self.sidebar_revision != 0 {
                    self.persist_sidebar(self.sidebar.preferred_width());
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
                self.appearance_focus = Some(appearance_focus::Choice::Material(material));
                self.appearance.theme = material.into();
                self.save_appearance()
            }
            Message::AppearanceMode(mode) => {
                self.appearance_focus = Some(appearance_focus::Choice::Mode(mode));
                self.appearance.mode = mode.into();
                self.save_appearance()
            }
            Message::Accent(accent) => {
                self.appearance_focus = Some(appearance_focus::Choice::Accent(accent));
                self.appearance.accent = accent.into();
                self.save_appearance()
            }
            Message::AppearancePointerPressed(id) => {
                if self.windows.is_library(id) {
                    self.appearance_focus = None;
                }
                Task::none()
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
            Message::DropboxLibraryRefreshed(id, result) => {
                let exists = self.sources.snapshot.as_ref().is_some_and(|snapshot| {
                    snapshot.libraries.iter().any(|library| library.id == id)
                });
                let reload = exists && result.is_ok();
                self.dropbox.apply_refresh(id, exists.then_some(result));
                if reload {
                    self.sources.reload()
                } else {
                    Task::none()
                }
            }
            Message::RequestSourceRemoval(id) => {
                self.sources.pending_remove = Some(id);
                Task::none()
            }
            Message::CancelSourceRemoval => {
                self.sources.pending_remove = None;
                Task::none()
            }
            Message::PlayIndexed(index) => self.play_indexed(CatalogueSelection::Source(index)),
            Message::PlayPlaylist(entry) => {
                self.play_indexed(CatalogueSelection::PlaylistEntry(&entry, false))
            }
            Message::StartPlaylist(playlist) => {
                let Some(entry) = self
                    .sources
                    .snapshot
                    .as_ref()
                    .filter(|snapshot| {
                        snapshot.selected_playlist.as_deref() == Some(playlist.as_str())
                    })
                    .and_then(|snapshot| {
                        snapshot
                            .playlist_entries
                            .iter()
                            .find(|entry| entry.track.is_some())
                    })
                    .map(|entry| entry.id.clone())
                else {
                    return Task::none();
                };
                self.play_indexed(CatalogueSelection::PlaylistEntry(&entry, true))
            }
            Message::PlaylistSearch(query) => {
                self.sources.playlist_search = query;
                Task::none()
            }
            Message::ShowPlaylistForm(visible) => {
                if self.sources.playlist_save_pending() {
                    return Task::none();
                }
                self.sources.show_playlist_form = visible;
                if visible {
                    return iced::widget::operation::focus("new-playlist-name");
                }
                self.sources.playlist_name.clear();
                Task::none()
            }
            Message::PlaylistName(name) => {
                if !self.sources.playlist_save_pending() {
                    self.sources.playlist_name = name;
                }
                Task::none()
            }
            Message::CreatePlaylist => {
                if self.sources.busy()
                    || self.sources.playlist_save_pending()
                    || self.sources.playlist_name.trim().is_empty()
                {
                    return Task::none();
                }
                self.sources.start(Operation::CreatePlaylist(
                    self.sources.playlist_name.clone(),
                ))
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
                    Ok(track) => {
                        if let Some(id) = track.indexed_id() {
                            self.queue_covers.remove(id);
                            if self.queue_covers.len() < row_cover::ROW_LIMIT
                                && let Some(handle) =
                                    track.artwork().and_then(row_cover::row_handle)
                            {
                                self.queue_covers.insert(id.to_owned(), handle);
                            }
                        }
                        self.transport(|session| {
                            session.replace_track(index, track)?;
                            session.select(index)
                        })
                    }
                    Err(error) => {
                        self.failed_track = Some(index);
                        if self.session.as_ref().and_then(Session::current).is_none() {
                            self.cover = Cover::from_track(
                                self.session
                                    .as_ref()
                                    .and_then(|session| session.tracks().get(index)),
                            );
                        }
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
                    tokio::task::spawn_blocking(move || row_cover::open_files(paths))
                        .await
                        .map_err(|error| format!("Could not read the selected tracks: {error}"))
                },
                Message::FilesRead,
            ),
            Message::FilesRead(Ok(files)) => {
                if let Some(session) = &mut self.session
                    && !files.tracks.is_empty()
                {
                    let first = session.tracks().len();
                    let remaining = row_cover::ROW_LIMIT.saturating_sub(self.direct_covers.len());
                    self.direct_covers
                        .extend(files.covers.into_iter().take(remaining));
                    session.extend(files.tracks);
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
                self.persist_sidebar(width);
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
                self.refresh_transport(Session::advance_if_finished, false)
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
                if self.windows.is_library(id) && self.page == LibraryPage::Settings {
                    let next = appearance_focus::Choice::next(self.appearance_focus, backwards);
                    self.appearance_focus = Some(next);
                    return next.take_focus();
                }
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
                if self.windows.is_library(id)
                    && self.page == LibraryPage::Settings
                    && let Some(choice) = self.appearance_focus
                {
                    return self.update(choice.message());
                }
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
                if self.windows.is_library(id)
                    && self.page == LibraryPage::Settings
                    && self.appearance_focus.is_some()
                {
                    return self.update(Message::ActivateMiniFocus(id));
                }
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

            Message::PlayRecorded(id, result) => {
                self.plays_pending = self.plays_pending.saturating_sub(1);
                match result {
                    Ok(count) => self.sources.confirm_play_count(&id, count),
                    Err(error) => {
                        self.error = Some(error);
                        self.close_after_history = None;
                        self.install_after_history = false;
                    }
                }
                if self.plays_pending == 0
                    && let Some(window) = self.close_after_history.take()
                {
                    return self.update(Message::CloseRequested(window));
                }
                if self.plays_pending == 0 && self.install_after_history {
                    return self.update(Message::ConfirmInstallUpdate);
                }
                Task::none()
            }
            Message::CloseRequested(id) => {
                if self.windows.is_library(id) && self.plays_pending > 0 {
                    self.close_after_history = Some(id);
                    return Task::none();
                }
                if self.windows.is_library(id) && self.sidebar_revision != 0 {
                    self.persist_sidebar(self.sidebar.preferred_width());
                }
                self.windows.close_requested(id)
            }
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

    fn persist_sidebar(&mut self, width: crate::appearance::sidebar::SidebarWidth) {
        if let Some(path) = self.sources.paths().map(|paths| paths.sidebar())
            && let Err(error) = crate::appearance::sidebar::save(&path, width)
        {
            self.error = Some(error.to_string());
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

    fn play_indexed(&mut self, selection: CatalogueSelection<'_>) -> Task<Message> {
        let Some(snapshot) = &self.sources.snapshot else {
            return Task::none();
        };
        let (position, tracks): (_, Vec<_>) = match selection {
            CatalogueSelection::Source(index) => (
                index,
                snapshot
                    .tracks
                    .iter()
                    .cloned()
                    .map(|track| (track, None))
                    .collect(),
            ),
            CatalogueSelection::PlaylistEntry(entry, restart) => {
                if !snapshot
                    .playlist_entries
                    .iter()
                    .any(|item| item.id == entry && item.track.is_some())
                {
                    return Task::none();
                }
                if !restart
                    && self
                        .session
                        .as_ref()
                        .and_then(Session::current)
                        .and_then(Track::playlist_entry)
                        == Some(entry)
                {
                    return self.update(Message::TogglePlayback);
                }
                let tracks: Vec<_> = snapshot
                    .playlist_entries
                    .iter()
                    .filter_map(|item| Some((item.track.clone()?, Some(item.id.clone()))))
                    .collect();
                let Some(position) = tracks
                    .iter()
                    .position(|(_, id)| id.as_deref() == Some(entry))
                else {
                    return Task::none();
                };
                (position, tracks)
            }
        };
        if position >= tracks.len() || self.session.is_none() {
            return Task::none();
        }
        let remote_libraries: std::collections::HashSet<_> = snapshot
            .libraries
            .iter()
            .filter(|library| library.kind == "dropbox")
            .map(|library| library.id.clone())
            .collect();
        let tracks: Vec<_> = tracks
            .into_iter()
            .map(|(track, entry)| {
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
                .with_playlist_entry(entry)
                .with_indexed_id(track.id)
                .with_library(track.library_id.clone());
                if is_remote {
                    item.with_remote(track.library_id, path)
                } else {
                    item
                }
            })
            .collect();
        self.direct_covers.clear();
        let mut seeded = std::collections::HashSet::new();
        self.queue_covers = tracks
            .iter()
            .filter_map(Track::indexed_id)
            .filter(|id| seeded.insert(*id))
            .filter_map(|id| Some((id.to_owned(), snapshot.covers.get(id)?.clone())))
            .take(row_cover::ROW_LIMIT)
            .collect();
        if let Some(session) = &mut self.session {
            session.replace(tracks);
        }
        if matches!(self.mini_focus, Some(MiniControl::Track(_))) {
            self.mini_focus = Some(MiniControl::Queue);
        }
        self.cover = Cover::default();
        self.select_track(position)
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
        let live: std::collections::HashSet<_> = session
            .tracks()
            .iter()
            .filter_map(Track::indexed_id)
            .collect();
        self.queue_covers.retain(|id, _| live.contains(id.as_str()));
        if session.current().is_none() {
            self.cover = Cover::default();
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
        let Some(mut track) = self
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
            if let Some(revision) = artwork_revision(&paths, &track)
                && let Some(bytes) =
                    crate::artwork_cache::ArtworkCache::new(paths.artwork_cache()).lookup(&revision)
            {
                track = track.with_artwork(bytes);
                if let Some(session) = &mut self.session {
                    let _ = session.replace_track(index, track.clone());
                    if session.current().is_none() {
                        self.cover = Cover::from_track(Some(&track));
                    }
                }
            }
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
            let paths = self.sources.paths();
            return Task::perform(
                async move {
                    tokio::task::spawn_blocking(move || prepare_local_track(paths.as_ref(), track))
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
        self.refresh_transport(operation, true)
    }

    fn refresh_transport(
        &mut self,
        operation: impl FnOnce(&mut Session) -> Result<(), SessionError>,
        clear_on_success: bool,
    ) -> Task<Message> {
        let mut started = None;
        if let Some(session) = &mut self.session {
            let current = session.current_index();
            let before = session.start_generation();
            match operation(session) {
                Ok(()) => {
                    if clear_on_success || current != session.current_index() {
                        self.error = None;
                    }
                    if session.is_playing() && before != session.start_generation() {
                        started = session.current().and_then(|track| {
                            Some((track.library()?.to_owned(), track.indexed_id()?.to_owned()))
                        });
                    }
                }
                Err(error) => self.error = Some(error.to_string()),
            }
            if current != session.current_index() {
                self.cover = Cover::from_track(session.current());
            }
        }
        started.map_or_else(Task::none, |(library, track)| {
            self.record_play(library, track)
        })
    }

    fn record_play(&mut self, library: String, track: String) -> Task<Message> {
        let Some(paths) = self.sources.paths() else {
            return Task::none();
        };
        self.plays_pending += 1;
        let event = uuid::Uuid::new_v4().to_string();
        let started = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_or(0, |duration| duration.as_secs() as i64);
        let id = track.clone();
        Task::perform(
            async move {
                tokio::task::spawn_blocking(move || {
                    persist_play(&paths, &event, &library, &track, started)
                })
                .await
                .map_err(|error| format!("Could not record playback: {error}"))?
                .map_err(|error| error.to_string())
            },
            move |result| Message::PlayRecorded(id, result),
        )
    }

    pub fn subscription(&self) -> Subscription<Message> {
        Subscription::batch([
            window::close_requests().map(Message::CloseRequested),
            iced::time::every(Duration::from_secs(1)).map(|_| Message::SourceWatchTick),
            window::resize_events().map(|(id, size)| Message::MiniResized(id, size)),
            if updates::installed_build() {
                iced::time::every(Duration::from_secs(6 * 60 * 60)).map(|_| Message::CheckUpdate)
            } else {
                Subscription::none()
            },
            if self.session.as_ref().is_some_and(Session::is_playing) {
                iced::time::every(Duration::from_millis(100)).map(|_| Message::Tick)
            } else {
                Subscription::none()
            },
            iced::event::listen_with(input_message),
        ])
    }

    pub fn title(&self, _id: window::Id) -> String {
        "Muzo".into()
    }

    pub fn view(&self, id: window::Id) -> Element<'_, Message> {
        if self.windows.is_mini(id) {
            view::mini(
                self.session.as_ref(),
                self.windows.mini_view(),
                (&self.cover, Some(&self.queue_covers), &self.direct_covers),
                &self.appearance,
                (self.error.as_deref(), self.failed_track.is_some()),
                self.mini_focus,
            )
        } else {
            view::library(
                &self.sources,
                &self.dropbox,
                (
                    &self.appearance,
                    &self.update_status,
                    &self.sidebar,
                    self.page,
                    self.appearance_focus,
                ),
                self.session.as_ref(),
                self.cover.thumbnail(),
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

fn input_message(
    event: iced::Event,
    status: iced::event::Status,
    id: window::Id,
) -> Option<Message> {
    if matches!(
        &event,
        iced::Event::Mouse(iced::mouse::Event::ButtonPressed(iced::mouse::Button::Left))
            | iced::Event::Touch(iced::touch::Event::FingerPressed { .. })
    ) {
        return Some(Message::AppearancePointerPressed(id));
    }
    if status == iced::event::Status::Captured {
        return None;
    }
    let iced::Event::Keyboard(iced::keyboard::Event::KeyPressed { key, modifiers, .. }) = event
    else {
        return None;
    };
    use iced::keyboard::{Key, key::Named};
    match (key.as_ref(), modifiers.command()) {
        (Key::Named(Named::Tab), false) => Some(Message::CycleMiniFocus(id, modifiers.shift())),
        (Key::Named(Named::Enter), false) => Some(Message::ActivateMiniFocus(id)),
        (Key::Named(Named::Space), false) => Some(Message::MiniSpace(id)),
        (Key::Named(Named::Escape), false) => Some(Message::ShowLibrary),
        (Key::Named(Named::ArrowLeft), false) => Some(Message::MiniArrow(id, -1)),
        (Key::Named(Named::ArrowRight), false) => Some(Message::MiniArrow(id, 1)),
        (Key::Character("m"), true) => Some(Message::ToggleMini(id)),
        (Key::Character("o"), true) => Some(Message::OpenFiles),
        (Key::Character("q"), true) if !cfg!(target_os = "macos") => Some(Message::ToggleQueue),
        (Key::Character("q"), true) if modifiers.shift() => Some(Message::ToggleQueue),
        _ => None,
    }
}

fn persist_play(
    paths: &CataloguePaths,
    event: &str,
    library: &str,
    track: &str,
    started: i64,
) -> Result<u32, crate::catalogue::CatalogueError> {
    paths
        .open()?
        .record_origin_play(event, library, track, started)
}

fn auto_update() -> Task<Message> {
    if updates::installed_build() {
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
    let track = track.with_audio(local, &metadata).without_artwork();
    cache_artwork(Some(paths), track, || {
        metadata.artwork().map(<[u8]>::to_vec)
    })
}

fn prepare_local_track(paths: Option<&CataloguePaths>, track: Track) -> Track {
    let path = track.path().to_owned();
    cache_artwork(paths, track, || {
        read_track(path).artwork().map(<[u8]>::to_vec)
    })
}

fn cache_artwork(
    paths: Option<&CataloguePaths>,
    track: Track,
    read: impl FnOnce() -> Option<Vec<u8>>,
) -> Track {
    let revision = paths.and_then(|paths| artwork_revision(paths, &track));
    let cover = match paths.zip(revision) {
        Some((paths, revision)) => crate::artwork_cache::ArtworkCache::new(paths.artwork_cache())
            .get_or_read(&revision, read),
        None => read().as_deref().and_then(crate::artwork_cache::thumbnail),
    };
    if let Some(bytes) = cover {
        return track.with_artwork(bytes);
    }
    track
}

fn artwork_revision(
    paths: &CataloguePaths,
    track: &Track,
) -> Option<crate::artwork_cache::Revision> {
    paths
        .open()
        .ok()?
        .artwork_revision(track.library()?, track.indexed_id()?)
        .ok()
}

#[cfg(test)]
#[path = "desktop/queue_tests.rs"]
mod queue_tests;
