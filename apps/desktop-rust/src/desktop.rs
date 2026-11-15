mod appearance_focus;
mod cover;
mod dropbox_panel;
pub(crate) mod library_sources;
pub(crate) mod material;
mod mini_focus;
mod playlist_picker_focus;
mod row_cover;
mod settings_scroll;
mod sidebar;
#[cfg(test)]
mod sidebar_tests;
pub use mini_focus::Control as MiniControl;
mod track_editor_focus;
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
use track_editor_focus::{Control as TrackEditorControl, Field as MetadataField};
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
    ClearSourceSearch,
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
    OpenPlaylistPicker(crate::catalogue::IndexedTrack),
    ClosePlaylistPicker,
    ChooseTrackPlaylist(String),
    ToggleTrackPlaylistPopup,
    DismissTrackPlaylistPopup,
    EscapePressed(window::Id),
    SubmitPlaylistPicker,
    EditField(MetadataField, String),
    TrackEditorTab(u64, bool, Option<MetadataField>),
    TrackEditorFocusApplied(u64),
    ActivateTrackEditor,
    TrackEditorFieldSubmit,
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
    ResizeSidebar(sidebar::Resize),
    Tick,
    ShowMini,
    ToggleMini(window::Id),
    MiniOpened(window::Id),
    LibraryVisibilityChecked(window::Id, bool),
    ShowLibrary,
    ToggleQueue,
    CycleMiniFocus(window::Id, bool),
    CycleFocusFromSidebarDivider(bool),
    ActivateMiniFocus(window::Id, bool),
    MiniSpace(window::Id, bool),
    MiniArrow(window::Id, i8),
    PlaylistPickerVerticalArrow(window::Id, i8),
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
    queue_resize_error: Option<String>,
    cover: Cover,
    direct_covers: row_cover::DirectCovers,
    queue_covers: row_cover::Covers,
    sources: Sources,
    dropbox: dropbox_panel::Settings,
    playback_request: u64,
    pending_track: Option<usize>,
    failed_track: Option<usize>,
    mini_focus: Option<mini_focus::Control>,
    editor_navigation: track_editor_focus::Navigation,
    sidebar: sidebar::Sidebar,
    page: LibraryPage,
    appearance_focus: Option<appearance_focus::Choice>,
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
                queue_resize_error: None,
                cover: Cover::default(),
                direct_covers: Default::default(),
                queue_covers: Default::default(),
                sources: Sources::new(None),
                dropbox: dropbox_panel::Settings::default(),
                playback_request: 0,
                pending_track: None,
                failed_track: None,
                mini_focus: None,
                editor_navigation: Default::default(),
                sidebar: sidebar::Sidebar::default(),
                page: LibraryPage::default(),
                appearance_focus: None,
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
                self.editor_navigation.reset();
                self.sources.close_track_panels();
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
            Message::ClearSourceSearch => {
                let _ = self.update(Message::SourceSearch(String::new()));
                iced::advanced::widget::operate(
                    iced::advanced::widget::operation::focusable::focus(view::SOURCE_SEARCH.into()),
                )
            }
            Message::SourceOperation(operation) => {
                if let Operation::Select(id) = &operation {
                    self.page = LibraryPage::Songs;
                    if self
                        .sources
                        .snapshot
                        .as_ref()
                        .and_then(|snapshot| snapshot.selected.as_ref())
                        != Some(id)
                    {
                        self.editor_navigation.reset();
                        self.sources.close_track_panels();
                    }
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
                if self.windows.is_library(id)
                    && let Some(editor) = &mut self.sources.editing
                {
                    editor.action_focus = None;
                    self.editor_navigation.reset();
                }
                if self.windows.is_library(id) {
                    self.appearance_focus = None;
                    if let Some(picker) = &mut self.sources.playlist_picker
                        && picker.preview.is_none()
                    {
                        picker.focus = None;
                    }
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
            Message::EditTrack(track) => {
                if self.sources.track_add_pending() {
                    return Task::none();
                }
                self.sources.playlist_picker = None;
                self.sources.playlist_message = None;
                self.editor_navigation.reset();
                self.sources.open_editor(track)
            }
            Message::OpenPlaylistPicker(track) => {
                self.editor_navigation.reset();
                self.sources.open_playlist_picker(track);
                Task::none()
            }
            Message::ClosePlaylistPicker => {
                if !self.sources.track_add_pending() {
                    self.sources.playlist_picker = None;
                }
                Task::none()
            }
            Message::ChooseTrackPlaylist(id) => {
                self.sources.choose_track_playlist(id);
                Task::none()
            }
            Message::ToggleTrackPlaylistPopup => self.toggle_playlist_popup(),
            Message::DismissTrackPlaylistPopup => {
                if let Some(picker) = &mut self.sources.playlist_picker {
                    picker.preview = None;
                }
                Task::none()
            }
            Message::SubmitPlaylistPicker => self.sources.submit_playlist_picker(),
            Message::EditField(field, value) => {
                if self.sources.editor_save_pending() {
                    return Task::none();
                }
                if let Some(editor) = &mut self.sources.editing {
                    editor.fields[field.index()] = value;
                }
                Task::none()
            }
            Message::TrackEditorTab(serial, backwards, field) => {
                if !self.editor_navigation.is_active(serial) {
                    return Task::none();
                }
                if self.page != LibraryPage::Songs || self.sources.editor_save_pending() {
                    self.editor_navigation.reset();
                    return Task::none();
                }
                let can_save = !self.sources.busy();
                let Some(editor) = self.sources.editing.as_mut() else {
                    self.editor_navigation.reset();
                    return Task::none();
                };
                let next = TrackEditorControl::next(
                    field.map(TrackEditorControl::Input).or(editor.action_focus),
                    backwards,
                    can_save,
                    can_save && editor.overridden,
                );
                editor.action_focus = next.message().map(|_| next);
                next.focus(serial)
            }
            Message::TrackEditorFocusApplied(serial) => self.editor_navigation.finish(serial),
            Message::ActivateTrackEditor => {
                let action = self.activate_track_editor();
                action.chain(self.editor_navigation.advance())
            }
            Message::TrackEditorFieldSubmit => {
                if self.editor_navigation.defer_activation() {
                    return Task::none();
                }
                self.save_track_edit(false)
            }
            Message::SaveTrackEdit(clear) => self.save_track_edit(clear),
            Message::CloseTrackEdit => {
                if !self.sources.editor_save_pending() {
                    self.editor_navigation.reset();
                    self.sources.close_editor();
                }
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
            Message::ResizeSidebar(action) => {
                if let Some(width) = self.sidebar.resize(action) {
                    self.persist_sidebar(width);
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
                self.refresh_transport(Session::advance_if_finished, false)
            }
            Message::ToggleMini(id) => {
                if self.windows.is_mini(id) {
                    self.windows.show_library()
                } else {
                    self.update(Message::ShowMini)
                }
            }
            Message::ShowMini => {
                self.mini_focus = None;
                if self.windows.mini_id().is_none() {
                    self.queue_resize_error = None;
                }
                self.windows.show_mini()
            }
            Message::CycleFocusFromSidebarDivider(backwards) => {
                if self.page == LibraryPage::Settings {
                    self.appearance_focus = Some(appearance_focus::Choice::Divider);
                }
                self.update(Message::CycleMiniFocus(
                    self.windows.library_id(),
                    backwards,
                ))
            }
            Message::CycleMiniFocus(id, backwards) => {
                if self.windows.is_library(id)
                    && self.page == LibraryPage::Songs
                    && let Some(task) = self.focus_playlist_picker(backwards)
                {
                    return task;
                }
                if self.windows.is_library(id)
                    && self.page == LibraryPage::Songs
                    && !self.sources.editor_save_pending()
                    && self.sources.editing.is_some()
                {
                    return self.editor_navigation.request(backwards);
                }
                if self.windows.is_library(id) && self.page == LibraryPage::Settings {
                    let next = appearance_focus::Choice::next(self.appearance_focus, backwards);
                    self.appearance_focus = Some(next);
                    return next.take_focus();
                }
                if self.windows.is_library(id) {
                    use iced::advanced::widget::{
                        operate,
                        operation::{focusable, scope},
                    };
                    let library = appearance_focus::LIBRARY.into();
                    return if backwards {
                        operate(scope(library, focusable::focus_previous::<()>())).discard()
                    } else {
                        operate(scope(library, focusable::focus_next::<()>())).discard()
                    };
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
            Message::ActivateMiniFocus(id, repeat) => {
                if let Some(task) = self.route_playlist_picker_activation(id, repeat) {
                    return task;
                }
                if let Some(task) = self.route_track_editor_activation(id) {
                    return task;
                }
                if self.windows.is_library(id)
                    && self.page == LibraryPage::Settings
                    && let Some(choice) = self.appearance_focus
                {
                    return choice
                        .message()
                        .map_or_else(Task::none, |message| self.update(message));
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
            Message::MiniSpace(id, repeat) => {
                if let Some(task) = self.route_playlist_picker_activation(id, repeat) {
                    return task;
                }
                if let Some(task) = self.route_track_editor_activation(id) {
                    return task;
                }
                if self.windows.is_library(id)
                    && self.page == LibraryPage::Settings
                    && self.appearance_focus.is_some()
                {
                    return self.update(Message::ActivateMiniFocus(id, repeat));
                }
                if self.windows.is_mini(id) && self.mini_focus.is_some() {
                    self.update(Message::ActivateMiniFocus(id, repeat))
                } else {
                    self.update(Message::TogglePlayback)
                }
            }
            Message::PlaylistPickerVerticalArrow(id, step) => self
                .route_playlist_picker_arrow(id, step)
                .unwrap_or_else(Task::none),
            Message::MiniArrow(id, step) => {
                if let Some(task) = self.route_playlist_picker_arrow(id, step) {
                    return task;
                }
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
            Message::EscapePressed(id) => {
                if self.windows.is_library(id)
                    && let Some(picker) = &mut self.sources.playlist_picker
                    && picker.preview.take().is_some()
                {
                    return Task::none();
                }
                self.windows.show_library()
            }
            Message::ShowLibrary => self.windows.show_library(),
            Message::ToggleQueue => {
                self.queue_resize_error = None;
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
                    self.queue_resize_error = Some(error.to_string());
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
                self.windows.close_requested(id)
            }
            Message::MiniResized(id, size) => {
                if self.windows.is_mini(id) {
                    self.windows.resized(id, size);
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

    fn focus_playlist_picker(&mut self, backwards: bool) -> Option<Task<Message>> {
        let pending = self.sources.track_add_pending();
        let can_add = !self.sources.busy();
        let picker = self.sources.playlist_picker.as_mut()?;
        if pending {
            return Some(Task::none());
        }
        let next = playlist_picker_focus::Control::next(
            picker.focus,
            backwards,
            picker.playlist.is_some(),
            can_add,
        );
        picker.focus = Some(next);
        picker.preview = None;
        Some(next.reveal())
    }

    fn route_playlist_picker_activation(
        &mut self,
        id: window::Id,
        repeat: bool,
    ) -> Option<Task<Message>> {
        if !self.windows.is_library(id) || self.page != LibraryPage::Songs {
            return None;
        }
        let picker = self.sources.playlist_picker.as_ref()?;
        if repeat && picker.focus == Some(playlist_picker_focus::Control::Choice) {
            return Some(Task::none());
        }
        let message = picker
            .focus
            .and_then(playlist_picker_focus::Control::message);
        Some(message.map_or_else(Task::none, |message| self.update(message)))
    }

    fn toggle_playlist_popup(&mut self) -> Task<Message> {
        if self.sources.track_add_pending() {
            return Task::none();
        }
        let Some(picker) = &mut self.sources.playlist_picker else {
            return Task::none();
        };
        picker.focus = Some(playlist_picker_focus::Control::Choice);
        if let Some(choice) = picker.preview.take() {
            self.sources.choose_track_playlist(choice);
            return Task::none();
        }
        picker.preview = picker.playlist.clone();
        picker
            .preview
            .as_deref()
            .map_or_else(Task::none, playlist_picker_focus::reveal_option)
    }

    fn route_playlist_picker_arrow(&mut self, id: window::Id, step: i8) -> Option<Task<Message>> {
        if !self.windows.is_library(id) || self.page != LibraryPage::Songs {
            return None;
        }
        let picker = self.sources.playlist_picker.as_ref()?;
        let reveal = (picker.focus == Some(playlist_picker_focus::Control::Choice))
            .then(|| self.sources.step_track_playlist(step))
            .flatten();
        Some(
            reveal
                .as_deref()
                .map_or_else(Task::none, playlist_picker_focus::reveal_option),
        )
    }

    fn route_track_editor_activation(&mut self, id: window::Id) -> Option<Task<Message>> {
        if !self.windows.is_library(id) || self.page != LibraryPage::Songs {
            return None;
        }
        let editor = self.sources.editing.as_ref()?;
        if self.editor_navigation.defer_activation() {
            return Some(Task::none());
        }
        editor.action_focus?;
        Some(self.activate_track_editor())
    }

    fn activate_track_editor(&mut self) -> Task<Message> {
        if self.page != LibraryPage::Songs || self.editor_navigation.focus_pending() {
            return Task::none();
        }
        let Some(editor) = self.sources.editing.as_ref() else {
            return Task::none();
        };
        let message = editor.action_focus.and_then(TrackEditorControl::message);
        if self.sources.editor_save_pending() {
            return Task::none();
        }
        message.map_or_else(Task::none, |message| self.update(message))
    }

    fn save_track_edit(&mut self, clear: bool) -> Task<Message> {
        if self.sources.busy() || self.sources.editor_save_pending() {
            return Task::none();
        }
        let Some(editor) = self.sources.editing.as_mut() else {
            return Task::none();
        };
        let non_empty = |field: MetadataField| {
            let value = editor.fields[field.index()].trim();
            (!value.is_empty()).then(|| value.to_owned())
        };
        let number = |index| {
            non_empty(index)
                .map(|value| value.parse::<i64>())
                .transpose()
        };
        let edit = if clear {
            crate::catalogue::MetadataOverride::default()
        } else {
            let (Ok(track_number), Ok(disc_number), Ok(year)) = (
                number(MetadataField::Track),
                number(MetadataField::Disc),
                number(MetadataField::Year),
            ) else {
                editor.error = Some("Track, disc and year must be whole numbers.".into());
                return Task::none();
            };
            if [track_number, disc_number]
                .into_iter()
                .flatten()
                .any(|value| value < 1)
            {
                editor.error = Some("Track and disc numbers must be at least 1.".into());
                return Task::none();
            }
            crate::catalogue::MetadataOverride {
                title: non_empty(MetadataField::Title),
                artist: non_empty(MetadataField::Artist),
                album: non_empty(MetadataField::Album),
                track_number,
                disc_number,
                genre: non_empty(MetadataField::Genre),
                year,
            }
        };
        editor.error = None;
        let operation =
            Operation::EditTrack(editor.track_id.clone(), editor.library_id.clone(), edit);
        self.sources.start(operation)
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
            let error = self.error.as_deref().or(self.queue_resize_error.as_deref());
            view::mini(
                self.session.as_ref(),
                self.windows.mini_view(),
                (&self.cover, Some(&self.queue_covers), &self.direct_covers),
                &self.appearance,
                (error, self.failed_track.is_some()),
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
    input_message_for_platform(event, status, id, cfg!(target_os = "macos"))
}

fn input_message_for_platform(
    event: iced::Event,
    status: iced::event::Status,
    id: window::Id,
    macos: bool,
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
    let iced::Event::Keyboard(iced::keyboard::Event::KeyPressed {
        key,
        modifiers,
        repeat,
        ..
    }) = event
    else {
        return None;
    };
    use iced::keyboard::{Key, Modifiers, key::Named};
    let command = if macos {
        modifiers.logo()
    } else {
        modifiers.control()
    };
    match (key.as_ref(), command) {
        (Key::Named(Named::Tab), false) => Some(Message::CycleMiniFocus(id, modifiers.shift())),
        (Key::Named(Named::Enter), false) => Some(Message::ActivateMiniFocus(id, repeat)),
        (Key::Named(Named::Space), false) => Some(Message::MiniSpace(id, repeat)),
        (Key::Named(Named::Escape), false) => Some(Message::EscapePressed(id)),
        (Key::Named(Named::ArrowLeft), false) => Some(Message::MiniArrow(id, -1)),
        (Key::Named(Named::ArrowRight), false) => Some(Message::MiniArrow(id, 1)),
        (Key::Named(Named::ArrowUp), false) => Some(Message::PlaylistPickerVerticalArrow(id, -1)),
        (Key::Named(Named::ArrowDown), false) => Some(Message::PlaylistPickerVerticalArrow(id, 1)),
        (Key::Character("m"), true) => Some(Message::ToggleMini(id)),
        (Key::Character("o"), true) => Some(Message::OpenFiles),
        (Key::Character("q"), true) if !macos => Some(Message::ToggleQueue),
        (Key::Character("l"), true)
            if macos && modifiers == (Modifiers::LOGO | Modifiers::SHIFT) =>
        {
            Some(Message::ToggleQueue)
        }
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
#[path = "desktop/cover_layout_tests.rs"]
mod cover_layout_tests;

#[cfg(test)]
#[path = "desktop/direct_file_row_tests.rs"]
mod direct_file_row_tests;

#[cfg(test)]
#[path = "desktop/library_track_layout_tests.rs"]
mod library_track_layout_tests;

#[cfg(test)]
#[path = "desktop/library_transport_layout_tests.rs"]
mod library_transport_layout_tests;

#[cfg(test)]
#[path = "desktop/transport_time_layout_tests.rs"]
mod transport_time_layout_tests;

#[cfg(test)]
#[path = "desktop/queue_tests.rs"]
mod queue_tests;

#[cfg(test)]
#[path = "desktop/queue_error_tests.rs"]
mod queue_error_tests;

#[cfg(test)]
#[path = "desktop/queue_shortcut_tests.rs"]
mod queue_shortcut_tests;

#[cfg(test)]
#[path = "desktop/saved_names_tests.rs"]
mod saved_names_tests;
