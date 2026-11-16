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
#[cfg(test)]
mod song_search_tests;
mod source_removal;

#[cfg(test)]
#[path = "desktop/source_removal_dialog_tests.rs"]
mod source_removal_dialog_tests;
#[cfg(test)]
mod source_removal_focus_tests;
#[cfg(test)]
#[path = "desktop/source_sidebar_layout_tests.rs"]
mod source_sidebar_layout_tests;
pub use mini_focus::Control as MiniControl;
mod track_editor_focus;
mod track_search;
pub use track_search::SongScope;
mod updates;
mod view;
#[cfg(test)]
mod window_chrome_tests;
mod windows;

use std::{ffi::OsString, path::PathBuf, time::Duration};

use iced::{Element, Subscription, Task, Theme, window};
use rodio::{DeviceSinkBuilder, MixerDeviceSink};

use crate::{
    artwork_cache::{ArtworkCache, Revision},
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
    TrackSearchText(String),
    TrackSearchScope(SongScope),
    TrackSearchShortcut(window::Id, bool),
    TrackSearchFinished(u64, Result<Vec<crate::catalogue::IndexedTrack>, String>),
    SourceRowClicked(String),
    OpenSourceSettings,
    LibraryPage(LibraryPage),
    SourceOperation(Operation),
    SourcesUpdated(Result<Snapshot, library_sources::UpdateError>),
    SourceWatchTick,
    ArtworkViewport(LibraryPage, std::ops::Range<usize>),
    QueueArtworkViewport(std::ops::Range<usize>),
    ArtworkReady(row_cover::Complete),
    QueueArtworkReconciled(row_cover::QueueReconciled),
    PlayRecorded(String, Result<u32, String>),
    Dropbox(dropbox_panel::Action),
    DropboxReady(Result<dropbox_panel::Snapshot, String>),
    DropboxLibraryRefreshed(String, Result<(String, usize), dropbox_panel::RefreshError>),
    Material(&'static str),
    AppearanceMode(&'static str),
    Accent(&'static str),
    AppearancePointerPressed(window::Id),
    SettingsFieldFocused(window::Id, iced::advanced::widget::Id),
    CheckUpdate,
    AppImageIntegration(Result<(), String>),
    UpdateReceived(updates::Status),
    DownloadUpdate(crate::distribution::Update),
    InstallUpdate(crate::distribution::Update, PathBuf),
    ConfirmInstallUpdate,
    CancelInstallUpdate,
    UpdateInstalled(Result<(), String>),
    RequestSourceRemoval(String),
    RequestSourceRemovalFromKeyboard(source_removal::Launcher),
    RevealSourceRemovalLauncher(source_removal::Launcher),
    CancelSourceRemoval,
    CycleSourceRemovalFocus,
    ActivateSourceRemovalFocus,
    SetSourceRemovalSpaceArmed(bool),
    ReleaseSourceRemovalSpace,
    ScrollSourceRemoval(source_removal::Scroll),
    PlayIndexed(usize),
    PlaySearchResult(String),
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
    RemoteCachedCoverReady(u64, Result<Option<CachedCover>, String>),
    RemoteTrackReady(u64, Result<PreparedTrack, String>),
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
    LibraryVisibilityKnown(window::Id, bool),
    WaylandMiniPrepared(window::Id, iced::Size, bool),
    WaylandLibraryResized(window::Id, iced::Size, bool),
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
    DragWindow(window::Id),
    #[cfg(not(target_os = "macos"))]
    ResizeWindow(window::Id, window::Direction),
    MinimizeWindow(window::Id),
    ToggleWindowMaximized(window::Id),
    MiniResized(window::Id, iced::Size),
    CheckQueueSize(window::Id, u64),
    QueueResizeTimedOut(window::Id, u64, iced::Size),
}

enum CatalogueSelection<'a> {
    Source(usize),
    GlobalSearch(usize),
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
    queue_artwork: row_cover::QueueArtwork,
    row_covers: row_cover::Loader,
    sources: Sources,
    dropbox: dropbox_panel::Settings,
    dropbox_request_focus: Option<dropbox_panel::Control>,
    playback_request: u64,
    pending_track: Option<usize>,
    failed_track: Option<usize>,
    mini_focus: Option<mini_focus::Control>,
    editor_navigation: track_editor_focus::Navigation,
    sidebar: sidebar::Sidebar,
    page: LibraryPage,
    appearance_focus: Option<appearance_focus::Choice>,
    plays_pending: usize,
    close_after_history: bool,
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
        let (mut desktop, open) = Self::with_windows(session, Windows::open());
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
        let files = launch_files(std::env::args_os().skip(1));
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

    #[cfg(test)]
    pub fn with_session(session: Result<Session, String>) -> (Self, Task<Message>) {
        Self::with_windows(session, Windows::open_for_test())
    }

    fn with_windows(
        session: Result<Session, String>,
        (windows, open): (Windows, Task<Message>),
    ) -> (Self, Task<Message>) {
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
                queue_artwork: Default::default(),
                row_covers: row_cover::Loader::default(),
                sources: Sources::new(None),
                dropbox: dropbox_panel::Settings::default(),
                dropbox_request_focus: None,
                playback_request: 0,
                pending_track: None,
                failed_track: None,
                mini_focus: None,
                editor_navigation: Default::default(),
                sidebar: sidebar::Sidebar::default(),
                page: LibraryPage::default(),
                appearance_focus: None,
                plays_pending: 0,
                close_after_history: false,
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
                        self.sources.folder_error =
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
                    self.sources.folder_error = None;
                }
                Task::none()
            }
            Message::FolderName(name) => {
                if self.sources.folder_editable() {
                    self.sources.folder_name = name;
                    self.sources.folder_error = None;
                }
                Task::none()
            }
            Message::FolderLocation(location) => {
                if self.sources.folder_editable() {
                    self.sources.folder_location = location;
                    self.sources.folder_error = None;
                }
                Task::none()
            }
            Message::SubmitFolder => {
                if !self.sources.can_submit_folder() {
                    return Task::none();
                }
                self.sources.start(Operation::AddNamedFolder(
                    self.sources.folder_name.clone(),
                    PathBuf::from(&self.sources.folder_location),
                ))
            }
            Message::LibraryPage(page) => {
                self.set_library_page(page);
                Task::batch([self.refresh_track_search(), self.load_row_covers()])
            }
            Message::OpenSourceSettings => {
                self.set_library_page(LibraryPage::Settings);
                settings_scroll::reveal_sources()
            }
            Message::SourceSearch(query) => {
                self.sources.search = query;
                Task::none()
            }
            Message::ClearSourceSearch => {
                let _ = self.update(Message::SourceSearch(String::new()));
                iced::advanced::widget::operate(
                    iced::advanced::widget::operation::focusable::focus(view::SOURCE_SEARCH.into()),
                )
            }
            Message::TrackSearchText(query) => {
                self.sources.close_track_panels();
                let paths = self.sources.paths();
                let task = self.sources.track_search.change(
                    self.page,
                    self.sources.snapshot.as_ref(),
                    query,
                    paths,
                );
                Task::batch([task, self.reset_search_scroll(), self.load_row_covers()])
            }
            Message::TrackSearchShortcut(id, global) => {
                if id != self.windows.library_id() {
                    return Task::none();
                }
                if global || self.page == LibraryPage::Settings {
                    return self.update(Message::TrackSearchScope(SongScope::AllSongs));
                }
                if self.page == LibraryPage::Songs {
                    let scope = if self
                        .sources
                        .snapshot
                        .as_ref()
                        .is_some_and(|snapshot| snapshot.selected.is_some())
                    {
                        SongScope::Source
                    } else {
                        SongScope::AllSongs
                    };
                    return self.update(Message::TrackSearchScope(scope));
                }
                iced::widget::operation::focus(view::TRACK_SEARCH)
            }
            Message::TrackSearchScope(scope) => {
                self.set_library_page(LibraryPage::Songs);
                self.sources.track_search.scope = scope;
                self.project_track_search();
                Task::batch([
                    self.refresh_track_search(),
                    self.reset_search_scroll(),
                    self.load_row_covers(),
                    iced::widget::operation::focus(view::TRACK_SEARCH),
                ])
            }
            Message::TrackSearchFinished(serial, result) => {
                if !self.sources.track_search.finish(serial, result) {
                    return Task::none();
                }
                self.project_track_search();
                self.load_row_covers()
            }
            Message::SourceRowClicked(id) => {
                if let Some(task) = self.sources.toggle_selection(&id) {
                    let selected = self
                        .sources
                        .snapshot
                        .as_ref()
                        .is_some_and(|snapshot| snapshot.selected.is_some());
                    self.sources.track_search.selected_source(selected);
                    self.set_library_page(LibraryPage::Songs);
                    task.chain(self.refresh_track_search())
                } else {
                    Task::none()
                }
            }
            Message::SourceOperation(Operation::Remove(id)) => self.sources.confirm_removal(&id),
            Message::SourceOperation(Operation::SelectPlaylist(id)) => {
                self.sources.track_search.clear_playlist();
                self.project_track_search();
                self.sources.start(Operation::SelectPlaylist(id))
            }
            Message::SourceOperation(operation) => self.sources.start(operation),
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
                let revisions = if result.is_ok() {
                    self.row_covers.catalogue_changed();
                    let pending = self.pending_track;
                    let failed = self.failed_track;
                    let playback = self.session.as_ref().into_iter().flat_map(|session| {
                        [
                            session.current(),
                            pending.and_then(|index| session.tracks().get(index)),
                            failed.and_then(|index| session.tracks().get(index)),
                        ]
                        .into_iter()
                        .flatten()
                        .chain(
                            session
                                .tracks()
                                .iter()
                                .filter(|track| track.artwork().is_some()),
                        )
                        .filter_map(Track::indexed_id)
                    });
                    let queued = self.queue_covers.keys().map(String::as_str).chain(playback);
                    self.queue_artwork.reconcile(self.sources.paths(), queued)
                } else {
                    Task::none()
                };
                let updated = result.is_ok();
                let reveal_added_folder = self.sources.apply(result);
                self.project_track_search();
                let refresh = if updated {
                    self.refresh_track_search()
                } else {
                    Task::none()
                };
                let artwork = if reveal_added_folder && self.page == LibraryPage::Settings {
                    self.update(Message::LibraryPage(LibraryPage::Songs))
                } else {
                    self.load_row_covers()
                };
                let queued = self.sources.start_pending().unwrap_or_else(Task::none);
                Task::batch([queued, artwork, revisions, refresh])
            }
            Message::QueueArtworkReconciled(result) => {
                let changed = self.queue_artwork.apply(result, &mut self.queue_covers);
                let preparation = self.refresh_stale_queue_artwork(&changed);
                Task::batch([preparation, self.load_row_covers()])
            }
            Message::ArtworkViewport(page, rows) => {
                if page != self.page || rows.is_empty() {
                    return Task::none();
                }
                if page == LibraryPage::Songs {
                    self.sources.song_viewport = Some(rows.clone());
                }
                self.row_covers.visible(page, rows);
                self.load_row_covers()
            }
            Message::QueueArtworkViewport(rows) => {
                if rows.is_empty() || !self.windows.queue_open() {
                    return Task::none();
                }
                self.row_covers.queue_visible(Some(rows));
                self.load_row_covers()
            }
            Message::ArtworkReady(cover) => {
                let queued = self.session.as_ref().is_some_and(|session| {
                    session
                        .tracks()
                        .iter()
                        .any(|track| track.indexed_id() == Some(cover.id.as_str()))
                });
                let from_queue = cover.target == row_cover::Target::Queue;
                let covers = if from_queue {
                    queued.then_some(&mut self.queue_covers)
                } else {
                    self.sources
                        .visible_snapshot_mut()
                        .map(|source| &mut source.covers)
                };
                let ready = self.row_covers.complete(cover, covers);
                if queued && let Some((id, loaded)) = ready {
                    let protect_download = self
                        .queue_artwork
                        .protects_download_cover(&id, &loaded.revision);
                    if !protect_download {
                        self.queue_artwork.remember(&id, loaded.revision);
                    }
                    if !from_queue && self.queue_covers.len() < row_cover::ROW_LIMIT {
                        self.queue_covers.insert(id.clone(), loaded.handle);
                    }
                    if !protect_download {
                        self.apply_completed_cover(&id, &loaded.bytes);
                    }
                }
                self.load_row_covers()
            }
            Message::SourceWatchTick => {
                let local = self.sources.poll();
                let remote = if self.dropbox.due() {
                    self.dropbox
                        .update(dropbox_panel::Action::Sync, self.sources.paths())
                } else {
                    Task::none()
                };
                Task::batch([local, remote, self.load_row_covers()])
            }
            Message::Dropbox(action) => {
                let selection_changed = matches!(action, dropbox_panel::Action::SelectFolder(..));
                let field = match &action {
                    dropbox_panel::Action::Begin => Some(dropbox_panel::Control::Code),
                    dropbox_panel::Action::CancelAuthorization => {
                        Some(dropbox_panel::Control::AppKey)
                    }
                    _ => None,
                };
                let request_focus = (self.page == LibraryPage::Settings && !self.dropbox.busy)
                    .then(|| self.dropbox.request_focus(&action))
                    .flatten();
                let task = self.dropbox.update(action, self.sources.paths());
                if self.dropbox.busy && request_focus.is_some() {
                    self.dropbox_request_focus = request_focus;
                }
                let focus = match field {
                    Some(control) if self.page == LibraryPage::Settings => {
                        self.focus_dropbox([control])
                    }
                    _ if selection_changed => self.restore_dropbox_focus(),
                    _ => Task::none(),
                };
                Task::batch([task, focus])
            }
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
                if self.windows.is_library(id) && self.sources.removal.is_some() {
                    return Task::none();
                }
                if self.windows.is_library(id)
                    && let Some(editor) = &mut self.sources.editing
                {
                    editor.action_focus = None;
                    self.editor_navigation.reset();
                }
                if self.windows.is_library(id) {
                    self.appearance_focus = None;
                    self.dropbox_request_focus = None;
                    if let Some(picker) = &mut self.sources.playlist_picker
                        && picker.preview.is_none()
                    {
                        picker.focus = None;
                    }
                    if self.page == LibraryPage::Settings {
                        return appearance_focus::focused_settings_field(id);
                    }
                }
                Task::none()
            }
            Message::SettingsFieldFocused(id, field) => {
                if self.windows.is_library(id)
                    && self.page == LibraryPage::Settings
                    && self.appearance_focus.is_none()
                {
                    self.appearance_focus = dropbox_panel::Control::focused_field(&field)
                        .map(appearance_focus::Choice::Dropbox);
                }
                Task::none()
            }
            Message::DropboxReady(mut result) => {
                let launcher = self.dropbox_request_focus.take().filter(|launcher| {
                    self.page == LibraryPage::Settings
                        && self.appearance_focus.as_ref().is_none_or(|choice| {
                            let appearance_focus::Choice::Dropbox(control) = choice else {
                                return false;
                            };
                            control == launcher
                                || (*launcher == dropbox_panel::Control::Connect
                                    && *control == dropbox_panel::Control::Code)
                        })
                });
                let entered_folder = result.as_ref().is_ok_and(|snapshot| {
                    snapshot.account == self.dropbox.snapshot.account
                        && !snapshot.path.is_empty()
                        && snapshot.path != self.dropbox.snapshot.path
                });
                if let Ok(snapshot) = &mut result {
                    for library in std::mem::take(&mut snapshot.disconnected_libraries) {
                        self.remove_queued_library(&library);
                    }
                }
                self.dropbox.apply(result);
                let reload = self.sources.reload();
                use dropbox_panel::Control;
                let focus = match launcher {
                    Some(Control::Connect) => {
                        self.focus_dropbox([Control::Code, Control::Browse, Control::AppKey])
                    }
                    Some(_) if entered_folder => self.focus_dropbox([
                        Control::SelectCurrent,
                        Control::Parent,
                        Control::Browse,
                    ]),
                    Some(control) => self.focus_dropbox([
                        control,
                        Control::Browse,
                        Control::Code,
                        Control::AppKey,
                    ]),
                    None => self.restore_dropbox_focus(),
                };
                Task::batch([reload, focus])
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
                self.sources.request_removal(&id, None);
                Task::none()
            }
            Message::RequestSourceRemovalFromKeyboard(launcher) => {
                self.sources
                    .request_removal(launcher.source_id(), Some(&launcher));
                Task::none()
            }
            Message::RevealSourceRemovalLauncher(launcher) => launcher.reveal(),
            Message::CancelSourceRemoval => self
                .sources
                .cancel_removal()
                .map_or_else(Task::none, |launcher| launcher.focus()),
            Message::CycleSourceRemovalFocus => {
                if let Some(removal) = &mut self.sources.removal {
                    removal.focus_next();
                }
                Task::none()
            }
            Message::ActivateSourceRemovalFocus => {
                if let Some(removal) = &self.sources.removal {
                    let message = if removal.cancel_focused {
                        Message::CancelSourceRemoval
                    } else {
                        Message::SourceOperation(Operation::Remove(removal.source.id.clone()))
                    };
                    self.update(message)
                } else {
                    Task::none()
                }
            }
            Message::SetSourceRemovalSpaceArmed(armed) => {
                if let Some(removal) = &mut self.sources.removal {
                    removal.set_space_armed(armed);
                }
                Task::none()
            }
            Message::ReleaseSourceRemovalSpace => {
                if self
                    .sources
                    .removal
                    .as_mut()
                    .is_some_and(source_removal::Removal::take_space_armed)
                {
                    self.update(Message::ActivateSourceRemovalFocus)
                } else {
                    Task::none()
                }
            }
            Message::ScrollSourceRemoval(scroll) => {
                if self.sources.removal.is_some() {
                    scroll.apply()
                } else {
                    Task::none()
                }
            }
            Message::PlayIndexed(index) => self.play_indexed(CatalogueSelection::Source(index)),
            Message::PlaySearchResult(id) => self.play_search_result(&id),
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
            Message::RemoteCachedCoverReady(request, result) => {
                if request != self.playback_request {
                    return Task::none();
                }
                let cover = match result {
                    Ok(cover) => cover,
                    Err(error) => {
                        eprintln!("Could not read cached Dropbox artwork: {error}");
                        return Task::none();
                    }
                };
                if let (Some(index), Some(cover), Some(session)) = (
                    self.pending_track.or(self.failed_track),
                    cover,
                    self.session.as_mut(),
                ) && let Some(track) = session.tracks().get(index)
                {
                    if let Some(id) = track.indexed_id() {
                        if self
                            .queue_artwork
                            .has_conflicting_revision(id, &cover.revision)
                            || self
                                .queue_artwork
                                .protects_download_cover(id, &cover.revision)
                        {
                            return Task::none();
                        }
                        self.queue_artwork.remember(id, cover.revision);
                    }
                    let track = track.clone().with_artwork(cover.bytes);
                    let _ = session.replace_track(index, track.clone());
                    if session.current().is_none() {
                        self.cover = Cover::from_track(Some(&track));
                    }
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
                    Ok(PreparedTrack {
                        track,
                        artwork_revision,
                        indexed_download_revision,
                    }) => {
                        let track = self.apply_prepared_cover(
                            index,
                            track,
                            artwork_revision,
                            indexed_download_revision,
                        );
                        let previous = self.session.as_ref().and_then(Session::current_index);
                        let playback = self.transport(|session| {
                            session.replace_track(index, track)?;
                            session.select(index)
                        });
                        if previous == Some(index) {
                            self.cover =
                                Cover::from_track(self.session.as_ref().and_then(Session::current));
                        }
                        playback
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
                    let next = appearance_focus::Choice::next(
                        self.appearance_focus.as_ref(),
                        backwards,
                        &self.sources,
                        &self.dropbox,
                    );
                    let task = next.take_focus();
                    self.appearance_focus = Some(next);
                    return task;
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
                    && let Some(choice) = &self.appearance_focus
                {
                    if repeat && matches!(choice, appearance_focus::Choice::Dropbox(_)) {
                        return Task::none();
                    }
                    return choice
                        .message(&self.dropbox)
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
            Message::MiniOpened(id) => {
                let opened = self.windows.mini_opened(id);
                if self.windows.is_mini(id) && self.windows.queue_open() {
                    Task::batch([opened, self.load_row_covers()])
                } else if self.windows.is_library(id)
                    && self.page == LibraryPage::Settings
                    && let Some(appearance_focus::Choice::Dropbox(control)) = &self.appearance_focus
                    && self.dropbox.focus_stops().contains(control)
                {
                    opened.chain(appearance_focus::Choice::Dropbox(control.clone()).take_focus())
                } else {
                    opened
                }
            }
            Message::LibraryVisibilityKnown(id, wayland) => {
                self.windows.library_visibility_known(id, wayland)
            }
            Message::WaylandMiniPrepared(id, size, maximized) => {
                self.windows.open_wayland_mini(id, size, maximized)
            }
            Message::WaylandLibraryResized(id, size, maximized) => {
                self.windows.record_library_size(id, size, maximized);
                Task::none()
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
                        self.close_after_history = false;
                        self.install_after_history = false;
                    }
                }
                if self.plays_pending == 0 && std::mem::take(&mut self.close_after_history) {
                    return iced::exit();
                }
                if self.plays_pending == 0 && self.install_after_history {
                    return self.update(Message::ConfirmInstallUpdate);
                }
                Task::none()
            }
            Message::CloseRequested(id) => {
                if self.windows.is_library(id) && self.plays_pending > 0 {
                    self.close_after_history = true;
                    return Task::none();
                }
                self.windows.close_requested(id)
            }
            Message::DragWindow(id) => window::drag(id),
            #[cfg(not(target_os = "macos"))]
            Message::ResizeWindow(id, direction) => window::drag_resize(id, direction),
            Message::MinimizeWindow(id) => window::minimize(id, true),
            Message::ToggleWindowMaximized(id) => window::toggle_maximize(id),
            Message::MiniResized(id, size) => {
                if self.windows.is_mini(id) {
                    let was_open = self.windows.queue_open();
                    self.windows.resized(id, size);
                    if was_open != self.windows.queue_open() {
                        return self.load_row_covers();
                    }
                } else {
                    return self.windows.library_resized(id, size);
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

    fn play_search_result(&mut self, id: &str) -> Task<Message> {
        let (Some(snapshot), Some(visible)) = (
            self.sources.snapshot.as_ref(),
            self.sources.track_search.projection.as_ref(),
        ) else {
            return Task::none();
        };
        if self.page != LibraryPage::Songs {
            return Task::none();
        }
        let Some(hit) = visible.tracks.iter().position(|track| track.id == id) else {
            return Task::none();
        };
        if self
            .sources
            .track_search
            .uses_global(self.page, Some(snapshot))
        {
            return self.play_indexed(CatalogueSelection::GlobalSearch(hit));
        }
        let Some(index) = snapshot.tracks.iter().position(|track| track.id == id) else {
            return Task::none();
        };
        self.play_indexed(CatalogueSelection::Source(index))
    }

    fn play_indexed(&mut self, selection: CatalogueSelection<'_>) -> Task<Message> {
        let Some(snapshot) = &self.sources.snapshot else {
            return Task::none();
        };
        let (position, tracks): (_, Vec<_>) = match selection {
            CatalogueSelection::Source(index) | CatalogueSelection::GlobalSearch(index) => {
                let rows = if matches!(selection, CatalogueSelection::GlobalSearch(_)) {
                    let Some(visible) = self.sources.track_search.projection.as_ref() else {
                        return Task::none();
                    };
                    &visible.tracks
                } else {
                    &snapshot.tracks
                };
                (
                    index,
                    rows.iter().cloned().map(|track| (track, None)).collect(),
                )
            }
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
        self.queue_artwork
            .replace(tracks.iter().map(|(track, _)| track));
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
        self.row_covers.queue_visible(None);
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
        self.queue_artwork.remove_library(library);
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
            let cover_paths = paths.clone();
            let cover_track = track.clone();
            let cover = Task::perform(
                async move {
                    tokio::task::spawn_blocking(move || {
                        let revision = artwork_revision(&cover_paths, &cover_track)?;
                        let bytes =
                            ArtworkCache::new(cover_paths.artwork_cache()).lookup(&revision)?;
                        Some(CachedCover { revision, bytes })
                    })
                    .await
                    .map_err(|error| error.to_string())
                },
                move |cover| Message::RemoteCachedCoverReady(request, cover),
            );
            let audio = Task::perform(
                async move {
                    tokio::task::spawn_blocking(move || {
                        let revision = artwork_revision(&paths, &track);
                        let download = dropbox_panel::remote_audio(&paths, &library, &path)
                            .map_err(|error| error.to_string())?;
                        Ok(prepare_downloaded_dropbox_track(
                            &paths, track, download, revision,
                        ))
                    })
                    .await
                    .map_err(|error| format!("Could not prepare the Dropbox track: {error}"))?
                },
                move |result| Message::RemoteTrackReady(request, result),
            );
            return Task::batch([cover, audio]);
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

    fn restore_dropbox_focus(&mut self) -> Task<Message> {
        use dropbox_panel::Control;

        if let Some(appearance_focus::Choice::Dropbox(control)) = self.appearance_focus.as_ref()
            && !self.dropbox.focus_stops().contains(control)
        {
            return self.focus_dropbox([
                Control::SelectCurrent,
                Control::Parent,
                Control::Browse,
                Control::Code,
                Control::AppKey,
            ]);
        }
        Task::none()
    }

    fn focus_dropbox(
        &mut self,
        targets: impl IntoIterator<Item = dropbox_panel::Control>,
    ) -> Task<Message> {
        let available = self.dropbox.focus_stops();
        let Some(control) = targets
            .into_iter()
            .find(|control| available.contains(control))
        else {
            return Task::none();
        };
        let choice = appearance_focus::Choice::Dropbox(control);
        let focus = choice.take_focus();
        self.appearance_focus = Some(choice);
        focus
    }

    fn set_library_page(&mut self, page: LibraryPage) {
        if self.page != page {
            self.sources.song_viewport = None;
        }
        self.page = page;
        self.appearance_focus = None;
        self.dropbox_request_focus = None;
        self.editor_navigation.reset();
        self.sources.close_track_panels();
        self.project_track_search();
    }

    fn project_track_search(&mut self) {
        self.sources
            .track_search
            .project(self.page, self.sources.snapshot.as_ref());
        if self.page == LibraryPage::Songs && self.sources.close_hidden_track_panels() {
            self.editor_navigation.reset();
        }
    }

    fn refresh_track_search(&mut self) -> Task<Message> {
        let paths = self.sources.paths();
        self.sources
            .track_search
            .refresh_global(self.page, self.sources.snapshot.as_ref(), paths)
    }

    fn reset_search_scroll(&mut self) -> Task<Message> {
        self.sources.song_viewport = None;
        self.row_covers.visible(self.page, 0..16);
        let id = match self.page {
            LibraryPage::Songs => track_editor_focus::SONGS_SCROLL,
            LibraryPage::Playlists => view::PLAYLIST_TRACKS_SCROLL,
            LibraryPage::Settings => return Task::none(),
        };
        iced::widget::operation::scroll_to(
            id,
            iced::widget::scrollable::AbsoluteOffset {
                x: None,
                y: Some(0.0),
            },
        )
    }

    fn apply_completed_cover(&mut self, id: &str, bytes: &[u8]) {
        let Some(session) = &mut self.session else {
            return;
        };
        let active = session.current_index();
        let prepared: Vec<_> = session
            .tracks()
            .iter()
            .enumerate()
            .filter(|(index, track)| {
                let downloaded = track.remote().is_some() && !track.path().as_os_str().is_empty();
                track.indexed_id() == Some(id)
                    && (Some(*index) == active
                        || Some(*index) == self.pending_track
                        || track.artwork().is_some()
                        || downloaded)
                    && track.artwork() != Some(bytes)
            })
            .map(|(index, track)| (index, track.clone().with_artwork(bytes.to_vec())))
            .collect();
        for (index, track) in prepared {
            if Some(index) == active {
                self.cover = Cover::from_track(Some(&track));
            }
            let _ = session.replace_track(index, track);
        }
    }

    fn apply_prepared_cover(
        &mut self,
        index: usize,
        track: Track,
        revision: Option<Revision>,
        indexed_download_revision: Option<Revision>,
    ) -> Track {
        let Some(id) = track.indexed_id() else {
            return track;
        };
        if let Some(indexed) = indexed_download_revision.as_ref()
            && !self.queue_artwork.has_conflicting_revision(id, indexed)
        {
            self.queue_artwork
                .keep_download_cover(id, indexed.clone(), revision);
            return track;
        }
        if let Some(result) = revision.as_ref().or(indexed_download_revision.as_ref())
            && self.queue_artwork.has_conflicting_revision(id, result)
        {
            let fresh = self
                .session
                .as_ref()
                .and_then(|session| session.tracks().get(index))
                .and_then(Track::artwork)
                .map(<[u8]>::to_vec);
            return match fresh {
                Some(bytes) => track.with_artwork(bytes),
                None => track.without_artwork(),
            };
        }
        self.queue_artwork.clear_download_cover(id);
        if let Some(revision) = revision {
            self.queue_artwork.remember(id, revision);
        }
        self.queue_covers.remove(id);
        if let Some(handle) = track.artwork().and_then(row_cover::row_handle) {
            self.row_covers.cover_available(id);
            if let Some(snapshot) = self.sources.visible_snapshot_mut() {
                row_cover::fill_snapshot_cover(snapshot, id, handle.clone());
            }
            if self.queue_covers.len() < row_cover::ROW_LIMIT {
                self.queue_covers.insert(id.to_owned(), handle);
            }
        }
        track
    }

    fn refresh_stale_queue_artwork(&mut self, changed: &[String]) -> Task<Message> {
        let Some(session) = &mut self.session else {
            return Task::none();
        };
        if changed.is_empty() {
            return Task::none();
        }
        let changed: std::collections::HashSet<_> = changed.iter().map(String::as_str).collect();
        let matches = |track: &Track| track.indexed_id().is_some_and(|id| changed.contains(id));
        let need_preparation = self
            .pending_track
            .filter(|index| session.tracks().get(*index).is_some_and(matches));
        let failed_revision = self
            .failed_track
            .is_some_and(|index| session.tracks().get(index).is_some_and(matches));
        if failed_revision {
            self.playback_request += 1;
        }
        if session.current().is_some_and(matches)
            || (session.current().is_none() && (need_preparation.is_some() || failed_revision))
        {
            self.cover = Cover::default();
        }
        let invalidated: Vec<_> = session
            .tracks()
            .iter()
            .enumerate()
            .filter(|(_, track)| track.artwork().is_some() && matches(track))
            .map(|(index, track)| (index, track.clone().without_artwork()))
            .collect();
        for (index, track) in invalidated {
            let _ = session.replace_track(index, track);
        }
        need_preparation.map_or_else(Task::none, |index| self.select_track(index))
    }

    fn load_row_covers(&mut self) -> Task<Message> {
        let Some(paths) = self.sources.paths() else {
            return Task::none();
        };
        if self.cover.thumbnail().is_none()
            && let Some(track) = self.session.as_ref().and_then(Session::current)
        {
            let playback = self
                .row_covers
                .request_playing(&paths, track, &mut self.queue_covers);
            if playback.units() > 0 {
                return playback;
            }
        }
        if self.windows.queue_open()
            && let Some(session) = &self.session
        {
            let queue =
                self.row_covers
                    .request_queue(&paths, session.tracks(), &mut self.queue_covers);
            if queue.units() > 0 {
                return queue;
            }
        }
        let Some(snapshot) = self.sources.visible_snapshot_mut() else {
            return Task::none();
        };
        self.row_covers.request(&paths, snapshot, self.page)
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
        let workspace = self.workspace(id);
        let content = if self.windows.is_library(id) {
            iced::widget::column![
                view::window_chrome::title_bar(
                    id,
                    &self.appearance,
                    iced::widget::Space::new().into(),
                ),
                workspace,
            ]
            .height(iced::Fill)
            .into()
        } else {
            workspace
        };
        #[cfg(not(target_os = "macos"))]
        let content = view::window_chrome::resize_edges(id, content);
        content
    }

    fn workspace(&self, id: window::Id) -> Element<'_, Message> {
        if self.windows.is_mini(id) {
            let error = self.error.as_deref().or(self.queue_resize_error.as_deref());
            view::mini(
                id,
                self.session.as_ref(),
                self.windows.mini_view(),
                (&self.cover, Some(&self.queue_covers), &self.direct_covers),
                &self.appearance,
                (error, self.failed_track.is_some()),
                self.mini_focus,
            )
        } else {
            let library = view::library(
                &self.sources,
                &self.dropbox,
                (
                    &self.appearance,
                    &self.update_status,
                    &self.sidebar,
                    self.page,
                    self.appearance_focus.as_ref(),
                ),
                self.session.as_ref(),
                self.cover.thumbnail(),
                (self.error.as_deref(), self.failed_track.is_some()),
            );
            view::source_removal_layer(library, self.sources.removal.as_ref(), &self.appearance)
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
    if command && !modifiers.alt() && !repeat && matches!(key.as_ref(), Key::Character("f" | "F")) {
        return Some(Message::TrackSearchShortcut(id, modifiers.shift()));
    }
    if status == iced::event::Status::Captured {
        return None;
    }
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

fn launch_files(arguments: impl IntoIterator<Item = OsString>) -> Vec<PathBuf> {
    let mut switches_allowed = true;
    arguments
        .into_iter()
        .filter_map(|argument| {
            if switches_allowed && argument == "--" {
                switches_allowed = false;
                return None;
            }
            let path = PathBuf::from(argument);
            // Electron forwards its switches when it relaunches an AppImage after an update.
            let switch = switches_allowed && path.as_os_str().as_encoded_bytes().starts_with(b"--");
            (!switch || path.is_file()).then_some(path)
        })
        .collect()
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

#[derive(Debug, Clone)]
pub struct CachedCover {
    revision: Revision,
    bytes: Vec<u8>,
}

#[derive(Debug, Clone)]
pub struct PreparedTrack {
    track: Track,
    artwork_revision: Option<Revision>,
    indexed_download_revision: Option<Revision>,
}

#[cfg(test)]
impl From<Track> for PreparedTrack {
    fn from(track: Track) -> Self {
        Self {
            track,
            artwork_revision: None,
            indexed_download_revision: None,
        }
    }
}

fn prepare_downloaded_dropbox_track(
    paths: &CataloguePaths,
    track: Track,
    download: crate::dropbox::cache::Download,
    indexed_revision: Option<Revision>,
) -> PreparedTrack {
    let downloaded_revision = download.metadata.as_ref().and_then(|metadata| {
        let (library, path) = track.remote()?;
        crate::catalogue::Catalogue::downloaded_artwork_revision(
            library,
            track.indexed_id()?,
            path,
            metadata.size,
            &metadata.rev,
        )
    });
    let (cache_revision, indexed_download_revision) = if indexed_revision == downloaded_revision {
        (indexed_revision, None)
    } else {
        (None, indexed_revision)
    };
    let mut prepared = prepare_dropbox_track(paths, track, download.path, cache_revision);
    prepared.artwork_revision = downloaded_revision;
    prepared.indexed_download_revision = indexed_download_revision;
    prepared
}

fn prepare_dropbox_track(
    paths: &CataloguePaths,
    track: Track,
    local: PathBuf,
    revision: Option<Revision>,
) -> PreparedTrack {
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
    let track = cache_artwork(Some(paths), revision.as_ref(), track, || {
        metadata.artwork().map(<[u8]>::to_vec)
    });
    PreparedTrack {
        track,
        artwork_revision: revision,
        indexed_download_revision: None,
    }
}

fn prepare_local_track(paths: Option<&CataloguePaths>, track: Track) -> PreparedTrack {
    let path = track.path().to_owned();
    let revision = paths.and_then(|paths| artwork_revision(paths, &track));
    let track = cache_artwork(paths, revision.as_ref(), track, || {
        read_track(path).artwork().map(<[u8]>::to_vec)
    });
    PreparedTrack {
        track,
        artwork_revision: revision,
        indexed_download_revision: None,
    }
}

fn cache_artwork(
    paths: Option<&CataloguePaths>,
    revision: Option<&Revision>,
    track: Track,
    read: impl FnOnce() -> Option<Vec<u8>>,
) -> Track {
    let cover = match paths.zip(revision) {
        Some((paths, revision)) => {
            ArtworkCache::new(paths.artwork_cache()).get_or_read(revision, read)
        }
        None => read().as_deref().and_then(crate::artwork_cache::thumbnail),
    };
    if let Some(bytes) = cover {
        return track.with_artwork(bytes);
    }
    track
}

fn artwork_revision(paths: &CataloguePaths, track: &Track) -> Option<Revision> {
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

#[cfg(test)]
#[test]
fn it_does_not_queue_electron_switches_after_upgrading_the_appimage() {
    let temporary = tempfile::tempdir().unwrap();
    let audio = temporary.path().join("--Set mix.mp3");
    std::fs::write(&audio, include_bytes!("../tests/fixtures/tone.mp3")).unwrap();
    let forwarded = [
        OsString::from("--no-sandbox"),
        OsString::from("--disable-gpu"),
        OsString::from("--user-data-dir=/fixture/Muzo"),
        OsString::from("--remote-debugging-port=0"),
        audio.clone().into_os_string(),
        OsString::from("--"),
        OsString::from("--Another track.flac"),
    ];
    assert_eq!(
        launch_files(forwarded),
        [audio, PathBuf::from("--Another track.flac")]
    );
}
