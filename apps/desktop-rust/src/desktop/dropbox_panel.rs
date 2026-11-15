use std::{path::PathBuf, sync::Arc, time::Duration};

use iced::Task;

use crate::{
    catalogue::{Catalogue, CatalogueError, Library},
    dropbox::{Authorization, Client, Credentials, DropboxError, Entry, cache},
};

use super::{Message, library_sources::CataloguePaths};

mod view;

#[derive(Debug, Clone)]
pub enum Action {
    Key(String),
    Code(String),
    Begin,
    Connect,
    CancelAuthorization,
    Disconnect,
    Browse(String),
    SelectFolder(String, String),
    AddSelected,
    AddFolders(String, Vec<(String, String)>),
    Interval(String),
    SaveInterval,
    Sync,
    SyncLibrary(String),
    OpenAuthorization(String),
}

#[derive(Debug, Clone, Default)]
pub struct Snapshot {
    pub account: Option<String>,
    pub app_key: String,
    pub reconnect: bool,
    pub interval: u16,
    pub path: String,
    pub folders: Vec<(String, String)>,
    pub disconnected_libraries: Vec<String>,
    pub added_folders: Vec<String>,
    pub added_paths: Vec<String>,
    pub folder_error: Option<String>,
}

pub struct Settings {
    pub busy: bool,
    pub error: Option<String>,
    pub app_key: String,
    pub code: String,
    pub interval: String,
    pub snapshot: Snapshot,
    selected_folders: Vec<(String, String)>,
    auth: Option<Arc<Authorization>>,
    connecting: bool,
    pub(super) refreshing: Option<String>,
    pub(super) refresh_status: Option<(String, String)>,
    next_sync: std::time::Instant,
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Dropbox(#[from] DropboxError),
    #[error(transparent)]
    Catalogue(#[from] CatalogueError),
    #[error("Enter a sync interval from 1 to 60 minutes.")]
    InvalidInterval,
}

#[derive(Debug, Clone, thiserror::Error)]
pub enum RefreshError {
    #[error("Dropbox authorization needs to be renewed for this account.")]
    Reconnect { account: String },
    #[error("{0}")]
    Failed(String),
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            busy: false,
            error: None,
            app_key: String::new(),
            code: String::new(),
            interval: "5".into(),
            snapshot: Snapshot::default(),
            selected_folders: Vec::new(),
            auth: None,
            connecting: false,
            refreshing: None,
            refresh_status: None,
            next_sync: std::time::Instant::now(),
        }
    }
}

impl Settings {
    pub fn update(&mut self, mut action: Action, paths: Option<CataloguePaths>) -> Task<Message> {
        match action {
            Action::SelectFolder(path, name) => {
                if !self.busy
                    && !self
                        .snapshot
                        .added_paths
                        .iter()
                        .any(|added| added.eq_ignore_ascii_case(&path))
                {
                    if let Some(index) = self
                        .selected_folders
                        .iter()
                        .position(|(selection, _)| selection.eq_ignore_ascii_case(&path))
                    {
                        self.selected_folders.remove(index);
                    } else {
                        self.selected_folders.push((path, name));
                    }
                }
                return Task::none();
            }
            Action::AddSelected => {
                if self.selected_folders.is_empty() {
                    return Task::none();
                }
                let Some(account) = &self.snapshot.account else {
                    return Task::none();
                };
                action = Action::AddFolders(account.clone(), self.selected_folders.clone());
            }
            Action::Key(value) => {
                if self.auth.is_none() {
                    self.app_key = value;
                }
                return Task::none();
            }
            Action::Code(value) => {
                self.code = value;
                return Task::none();
            }
            Action::Interval(value) => {
                self.interval = value;
                return Task::none();
            }
            Action::OpenAuthorization(url) => {
                if let Err(error) = open::that_detached(url) {
                    self.error = Some(format!("Could not start the browser: {error}"));
                }
                return Task::none();
            }
            _ => {}
        }
        if self.busy {
            return Task::none();
        }
        if matches!(action, Action::CancelAuthorization) {
            self.auth = None;
            self.code.clear();
            self.error = None;
            return Task::none();
        }
        if matches!(action, Action::Begin) {
            let key = self.app_key.trim();
            if key.is_empty() {
                return Task::none();
            }
            self.error = None;
            return match Authorization::new(key).and_then(|authorization| {
                let url = authorization.url()?;
                self.auth = Some(Arc::new(authorization));
                self.code.clear();
                Ok(url)
            }) {
                Ok(url) => Task::done(Message::Dropbox(Action::OpenAuthorization(url.to_string()))),
                Err(error) => {
                    self.error = Some(error.to_string());
                    Task::none()
                }
            };
        }
        let Some(paths) = paths else {
            return Task::none();
        };
        if let Action::SyncLibrary(id) = action {
            self.busy = true;
            self.refreshing = Some(id.clone());
            self.refresh_status = None;
            let result_id = id.clone();
            return Task::perform(
                async move {
                    tokio::task::spawn_blocking(move || refresh_library(&paths, &id))
                        .await
                        .map_err(|error| {
                            RefreshError::Failed(format!(
                                "Could not refresh the Dropbox library: {error}"
                            ))
                        })?
                },
                move |result| Message::DropboxLibraryRefreshed(result_id.clone(), result),
            );
        }
        self.next_sync = std::time::Instant::now()
            + Duration::from_secs(u64::from(self.snapshot.interval.max(1)) * 60);
        self.busy = true;
        self.error = None;
        self.connecting = matches!(action, Action::Connect);
        let auth = self.auth.clone();
        let code = if self.connecting {
            std::mem::take(&mut self.code)
        } else {
            String::new()
        };
        let interval = self.interval.clone();
        let old = self.snapshot.clone();
        Task::perform(
            async move {
                tokio::task::spawn_blocking(move || {
                    match run(
                        &paths,
                        action,
                        auth.as_deref(),
                        &code,
                        &interval,
                        old.clone(),
                    ) {
                        Ok(snapshot) => Ok(snapshot),
                        Err(Error::Dropbox(DropboxError::Reconnect)) => Ok(Snapshot {
                            reconnect: true,
                            ..old
                        }),
                        Err(error) => Err(error.to_string()),
                    }
                })
                .await
                .map_err(|error| format!("Dropbox request failed: {error}"))?
            },
            Message::DropboxReady,
        )
    }

    pub fn due(&self) -> bool {
        !self.busy && std::time::Instant::now() >= self.next_sync
    }

    pub fn can_refresh(&self) -> bool {
        !self.busy
    }

    pub fn apply_refresh(
        &mut self,
        id: String,
        result: Option<Result<(String, usize), RefreshError>>,
    ) {
        if self.refreshing.as_deref() != Some(&id) {
            return;
        }
        self.busy = false;
        self.refreshing = None;
        let Some(result) = result else {
            return;
        };
        let status = match result {
            Ok((name, tracks)) => {
                let unit = if tracks == 1 { "track" } else { "tracks" };
                format!("Refreshed {name}: {tracks} {unit}.")
            }
            Err(error) => {
                if let RefreshError::Reconnect { account } = &error
                    && self.snapshot.account.as_ref() == Some(account)
                {
                    self.snapshot.reconnect = true;
                }
                error.to_string()
            }
        };
        self.refresh_status = Some((id, status));
    }

    pub fn apply(&mut self, result: Result<Snapshot, String>) {
        self.busy = false;
        let connecting = std::mem::take(&mut self.connecting);
        match result {
            Ok(mut snapshot) => {
                if self.snapshot.account != snapshot.account {
                    self.selected_folders.clear();
                    snapshot.path.clear();
                    snapshot.folders.clear();
                }
                self.selected_folders.retain(|(path, _)| {
                    !snapshot
                        .added_folders
                        .iter()
                        .any(|added| added.eq_ignore_ascii_case(path))
                });
                if let Some(error) = &snapshot.folder_error {
                    self.error = Some(error.clone());
                }
                if self.app_key.is_empty() {
                    self.app_key = snapshot.app_key.clone();
                }
                self.interval = snapshot.interval.to_string();
                self.next_sync = std::time::Instant::now()
                    + Duration::from_secs(u64::from(snapshot.interval.max(1)) * 60);
                if connecting && snapshot.account.is_some() && !snapshot.reconnect {
                    self.auth = None;
                    self.next_sync = std::time::Instant::now();
                }
                self.snapshot = snapshot;
            }
            Err(error) => {
                self.error = Some(error);
            }
        }
    }
}

fn run(
    paths: &CataloguePaths,
    action: Action,
    auth: Option<&Authorization>,
    code: &str,
    interval: &str,
    mut snapshot: Snapshot,
) -> Result<Snapshot, Error> {
    let mut catalogue = paths.open()?;
    let client = Client::new()?;
    if let Action::Connect = &action {
        let auth = auth.ok_or(DropboxError::Reconnect)?;
        let token = client.exchange(auth, code)?;
        let (Some(account), Some(refresh)) = (token.account_id, token.refresh_token) else {
            return Err(DropboxError::InvalidAccount.into());
        };
        Credentials::connect(&catalogue, &account, auth.client_id(), &refresh)?;
    }
    if let Action::SaveInterval = &action {
        let minutes = parse_interval(interval).ok_or(Error::InvalidInterval)?;
        catalogue.set_dropbox_interval(minutes)?;
    }

    let previous_account = snapshot.account.clone();
    let current = catalogue.latest_dropbox()?;
    snapshot.disconnected_libraries.clear();
    snapshot.added_folders.clear();
    snapshot.added_paths.clear();
    snapshot.folder_error = None;
    if let Some((account, _)) = &current
        && matches!(action, Action::Disconnect)
    {
        snapshot.disconnected_libraries = Credentials::disconnect(&catalogue, account)?;
        snapshot.folders.clear();
        snapshot.path.clear();
    }
    snapshot.interval = catalogue.dropbox_interval()?;
    snapshot.account = None;
    snapshot.reconnect = false;
    let Some((account, app_key)) = catalogue.latest_dropbox()? else {
        return Ok(snapshot);
    };
    snapshot.app_key = app_key.clone();
    snapshot.account = Some(account.clone());
    let owned = catalogue.dropbox_libraries_for(&account)?;
    snapshot.added_paths = catalogue
        .libraries()?
        .into_iter()
        .filter(|library| owned.contains(&library.id))
        .map(|library| library.location.to_string_lossy().into_owned())
        .collect();
    if matches!(action, Action::Disconnect | Action::Connect) {
        return Ok(snapshot);
    }
    let refresh = match Credentials::read_or_migrate(&catalogue, &account) {
        Ok(Some(refresh)) => refresh,
        Ok(None) | Err(DropboxError::LegacyCredential) => {
            snapshot.reconnect = true;
            return Ok(snapshot);
        }
        Err(error) => return Err(error.into()),
    };

    let refresh_current = || client.refresh(&app_key, &refresh);
    match action {
        Action::Browse(path) => {
            if previous_account
                .as_ref()
                .is_some_and(|previous| previous != &account)
            {
                snapshot.folder_error =
                    Some("The connected Dropbox account changed. Browse again.".into());
                return Ok(snapshot);
            }
            let token = match refresh_current() {
                Ok(token) => token,
                Err(DropboxError::Reconnect) => {
                    snapshot.reconnect = true;
                    return Ok(snapshot);
                }
                Err(error) => return Err(error.into()),
            };
            snapshot.folders = client
                .list(&token.access_token, &path, false)?
                .into_iter()
                .filter(|entry| entry.kind == "folder")
                .filter_map(|entry| Some((entry.path()?.to_owned(), entry.name)))
                .collect();
            snapshot.path = path;
        }
        Action::AddFolders(expected, folders) => {
            if expected != account {
                snapshot.folder_error =
                    Some("The connected Dropbox account changed. Browse again.".into());
                return Ok(snapshot);
            }
            let token = match refresh_current() {
                Ok(token) => token,
                Err(DropboxError::Reconnect) => {
                    snapshot.reconnect = true;
                    return Ok(snapshot);
                }
                Err(error) => return Err(error.into()),
            };
            let mut failures = Vec::new();
            for (path, name) in folders {
                let result = (|| -> Result<(), Error> {
                    let entries = client.list(&token.access_token, &path, true)?;
                    catalogue.add_dropbox_indexed(&account, &path, &name, &entries)?;
                    Ok(())
                })();
                match result {
                    Ok(()) => {
                        snapshot.added_paths.push(path.clone());
                        snapshot.added_folders.push(path);
                    }
                    Err(error) => failures.push(format!("{name}: {error}")),
                }
            }
            if !failures.is_empty() {
                snapshot.folder_error = Some(failures.join("; "));
            }
        }
        Action::Sync => {
            for library in catalogue
                .libraries()?
                .into_iter()
                .filter(|library| library.kind == "dropbox")
            {
                if let Some(owner) = catalogue.dropbox_account_for(&library.id)? {
                    let Some(key) = catalogue.dropbox_client(&owner)? else {
                        continue;
                    };
                    let refresh = match Credentials::read_or_migrate(&catalogue, &owner) {
                        Ok(Some(refresh)) => refresh,
                        Ok(None) => continue,
                        Err(DropboxError::LegacyCredential | DropboxError::Credential(_))
                            if owner != account =>
                        {
                            continue;
                        }
                        Err(error) => return Err(error.into()),
                    };
                    let token = match client.refresh(&key, &refresh) {
                        Ok(token) => token,
                        Err(DropboxError::Reconnect) if owner == account => {
                            snapshot.reconnect = true;
                            continue;
                        }
                        Err(DropboxError::Reconnect) => continue,
                        Err(error) => return Err(error.into()),
                    };
                    let entries = list_library(&client, &token.access_token, &library)?;
                    catalogue.sync_dropbox(&library.id, &entries)?;
                }
            }
        }
        _ => {}
    }
    Ok(snapshot)
}

fn parse_interval(value: &str) -> Option<u16> {
    value
        .trim()
        .parse()
        .ok()
        .filter(|minutes| (1..=60).contains(minutes))
}

fn folder_name(path: &str) -> String {
    path.rsplit('/')
        .find(|name| !name.is_empty())
        .unwrap_or("Dropbox")
        .to_owned()
}

fn list_library(
    client: &Client,
    token: &str,
    library: &Library,
) -> Result<Vec<Entry>, DropboxError> {
    client.list(token, &library.location.to_string_lossy(), true)
}

fn refresh_library(paths: &CataloguePaths, id: &str) -> Result<(String, usize), RefreshError> {
    let fail = |error: CatalogueError| RefreshError::Failed(error.to_string());
    let mut catalogue = paths.open().map_err(fail)?;
    let library = catalogue
        .libraries()
        .map_err(fail)?
        .into_iter()
        .find(|library| library.id == id && library.kind == "dropbox")
        .ok_or_else(|| fail(CatalogueError::UnknownFolder))?;
    let account = catalogue
        .dropbox_account_for(id)
        .map_err(fail)?
        .ok_or_else(|| RefreshError::Failed(DropboxError::Reconnect.to_string()))?;
    let remote_error = |error: DropboxError| match error {
        DropboxError::Reconnect | DropboxError::LegacyCredential => RefreshError::Reconnect {
            account: account.clone(),
        },
        other => RefreshError::Failed(other.to_string()),
    };
    let (key, refresh) = account_credentials(&catalogue, &account).map_err(remote_error)?;
    let client = Client::new().map_err(remote_error)?;
    let token = client.refresh(&key, &refresh).map_err(remote_error)?;
    let entries = list_library(&client, &token.access_token, &library).map_err(remote_error)?;
    let tracks = catalogue.sync_dropbox(id, &entries).map_err(fail)?;
    Ok((library.name, tracks))
}

fn account_credentials(
    catalogue: &Catalogue,
    account: &str,
) -> Result<(String, String), DropboxError> {
    let key = catalogue
        .dropbox_client(account)?
        .ok_or(DropboxError::Reconnect)?;
    let refresh =
        Credentials::read_or_migrate(catalogue, account)?.ok_or(DropboxError::Reconnect)?;
    Ok((key, refresh))
}

pub fn remote_audio(paths: &CataloguePaths, library: &str, path: &str) -> Result<PathBuf, Error> {
    let catalogue: Catalogue = paths.open()?;
    let account = catalogue
        .dropbox_account_for(library)?
        .ok_or(DropboxError::Reconnect)?;
    let (client_id, refresh) = account_credentials(&catalogue, &account)?;
    let client = Client::new()?;
    let token = client.refresh(&client_id, &refresh)?;
    Ok(cache::download(
        &client,
        &token.access_token,
        &paths.dropbox_cache(),
        &account,
        path,
    )?)
}

#[cfg(test)]
mod tests {
    use super::*;

    use iced::{
        Element, Font, Pixels, Rectangle, Renderer, Size,
        advanced::{
            Layout, layout,
            widget::{Id, Operation, Tree},
        },
    };

    #[derive(Default)]
    struct Labels(std::collections::HashMap<String, Rectangle>);

    impl Operation for Labels {
        fn traverse(&mut self, visit: &mut dyn FnMut(&mut dyn Operation)) {
            visit(self);
        }

        fn text(&mut self, _: Option<&Id>, bounds: Rectangle, value: &str) {
            self.0.insert(value.into(), bounds);
        }
    }

    fn rendered_labels(settings: &Settings, width: f32) -> Labels {
        let appearance = crate::appearance::Appearance::default();
        let mut element: Element<'_, Message> = settings.view(appearance.surfaces()).into();
        let mut tree = Tree::new(&element);
        let renderer = Renderer::new(Font::DEFAULT, Pixels(16.0));
        let size = Size::new(width, 660.0);
        let widget = element.as_widget_mut();
        let node = widget.layout(&mut tree, &renderer, &layout::Limits::new(Size::ZERO, size));
        let mut labels = Labels::default();
        widget.operate(&mut tree, Layout::new(&node), &renderer, &mut labels);
        labels
    }

    #[test]
    fn it_labels_sync_controls_and_fits_the_folder_name_at_a_narrow_width() {
        let folder = "A long folder name for a whole record collection";
        let mut settings = Settings::default();
        settings.snapshot.account = Some("fixture".into());
        settings.snapshot.path = "/Music".into();
        settings.snapshot.folders = vec![("/Music/Long".into(), folder.into())];
        let labels = rendered_labels(&settings, 318.0);
        assert!(labels.0.contains_key("Sync now"));
        assert!(labels.0.contains_key("Sync interval"));
        assert!(labels.0.contains_key("minutes"));
        assert!(labels.0.contains_key("Dropbox / Music"));
        assert!(labels.0.contains_key("Parent folder"));
        let name = labels
            .0
            .get(folder)
            .expect("the folder name should be visible");
        assert!(name.width > 0.0 && name.x + name.width <= 318.0);
        assert!(
            !labels
                .0
                .keys()
                .any(|label| label == &format!("Select {folder}"))
        );
    }

    #[test]
    fn it_shows_all_selected_paths_next_to_the_dropbox_add_action() {
        let mut settings = Settings::default();
        settings.snapshot.account = Some("fixture".into());
        settings.snapshot.path = "/Library".into();
        for number in 1..=12 {
            let name = format!("Folder {number}");
            settings
                .snapshot
                .folders
                .push((format!("/Library/{name}"), name));
        }
        settings.selected_folders = vec![
            ("/Library/Folder 12".into(), "Folder 12".into()),
            ("/Archive/Folder 12".into(), "Folder 12".into()),
        ];

        let labels = rendered_labels(&settings, 318.0);
        for label in [
            "/Library/Folder 12",
            "/Archive/Folder 12",
            "Remove",
            "Add 2 folders",
        ] {
            assert!(
                labels.0.contains_key(label),
                "missing from selection summary: {label}"
            );
        }
    }

    #[test]
    fn it_keeps_pending_authorization_visible_while_a_connected_account_is_displayed() {
        let mut settings = Settings::default();
        settings.snapshot.account = Some("existing".into());
        settings.app_key = "replacement-key".into();
        settings.auth = Some(Arc::new(Authorization::new(&settings.app_key).unwrap()));
        let labels = rendered_labels(&settings, 350.0);
        for expected in [
            "Connected to Dropbox",
            "Browse folders",
            "Authorization code",
            "Finish connecting",
            "Restart authorization",
            "Cancel",
        ] {
            assert!(
                labels.0.contains_key(expected),
                "missing authorization action: {expected}"
            );
        }
    }

    #[test]
    fn it_reserves_a_gutter_for_the_scrollbar_inside_a_long_folder_list() {
        const WIDTH: f32 = 318.0;
        const ARROW_RIGHT_GUTTER: f32 = 22.0;
        let mut settings = Settings::default();
        settings.snapshot.account = Some("account".into());
        settings.snapshot.path = "/Music".into();
        for index in 0..12 {
            let name = format!("Album {index}");
            settings
                .snapshot
                .folders
                .push((format!("/Music/{name}"), name));
        }
        let labels = rendered_labels(&settings, WIDTH);
        let chevron = labels
            .0
            .get("›")
            .expect("the folder should have a navigation cue");
        assert!(
            chevron.x + chevron.width <= WIDTH - ARROW_RIGHT_GUTTER,
            "the scrollbar covers the folder navigation cue: {chevron:?}"
        );
    }

    #[test]
    fn it_reopens_authorization_when_the_displayed_account_needs_reconnecting() {
        let mut settings = Settings {
            snapshot: Snapshot {
                account: Some("displayed".into()),
                ..Default::default()
            },
            ..Default::default()
        };

        settings.refreshing = Some("source".into());
        settings.apply_refresh(
            "source".into(),
            Some(Err(RefreshError::Reconnect {
                account: "displayed".into(),
            })),
        );

        assert!(settings.snapshot.reconnect);
    }

    #[test]
    fn it_reports_when_the_requested_dropbox_source_owner_has_disconnected() {
        let directory = tempfile::tempdir().unwrap();
        let paths = CataloguePaths::in_directory(directory.path());
        let mut catalogue = paths.open().unwrap();
        catalogue.save_dropbox_account("previous", "key").unwrap();
        let library = catalogue
            .add_dropbox("previous", "/Archive", "Archive")
            .unwrap();
        catalogue.disconnect_dropbox("previous").unwrap();

        let result = refresh_library(&paths, &library.id);

        assert!(matches!(
            result,
            Err(RefreshError::Reconnect { account }) if account == "previous"
        ));
    }

    #[test]
    fn it_leaves_the_background_schedule_and_pending_drafts_alone_on_source_refresh() {
        let directory = tempfile::tempdir().unwrap();
        let paths = CataloguePaths::in_directory(directory.path());
        let mut settings = Settings {
            code: "pending-code".into(),
            interval: "30".into(),
            ..Default::default()
        };
        let scheduled = settings.next_sync;

        let refresh = settings.update(Action::SyncLibrary("source".into()), Some(paths));

        assert_ne!(refresh.units(), 0);
        assert_eq!(settings.code, "pending-code");
        assert_eq!(settings.interval, "30");
        assert_eq!(settings.next_sync, scheduled);
    }

    #[test]
    fn it_disconnects_the_account_while_a_separate_authorization_is_pending() {
        let directory = tempfile::tempdir().unwrap();
        let paths = CataloguePaths::in_directory(directory.path());
        let mut settings = Settings::default();
        let _ = settings.update(Action::Key("replacement-key".into()), None);
        let _ = settings.update(Action::Begin, None);
        settings.apply(Ok(Snapshot {
            account: Some("connected".into()),
            ..Default::default()
        }));

        let disconnect = settings.update(Action::Disconnect, Some(paths));

        assert!(settings.busy);
        assert_ne!(disconnect.units(), 0);
    }

    #[test]
    fn it_cancels_pending_dropbox_authorization_and_accepts_a_different_key() {
        let mut settings = Settings::default();
        let _ = settings
            .update(Action::Key("first-key".into()), None)
            .discard::<()>();
        let _ = settings.update(Action::Begin, None).discard::<()>();
        let _ = settings
            .update(Action::Code("first-code".into()), None)
            .discard::<()>();

        let _ = settings.update(Action::CancelAuthorization, None);
        let _ = settings
            .update(Action::Key("second-key".into()), None)
            .discard::<()>();

        assert!(settings.auth.is_none());
        assert!(settings.code.is_empty());
        assert_eq!(settings.app_key, "second-key");
    }

    #[test]
    fn it_clears_pending_authorization_after_connecting() {
        let directory = tempfile::tempdir().unwrap();
        let paths = CataloguePaths::in_directory(directory.path());
        let mut settings = Settings::default();
        let _ = settings
            .update(Action::Key("test-key".into()), None)
            .discard::<()>();
        let _ = settings.update(Action::Begin, None).discard::<()>();
        let _ = settings
            .update(Action::Code("test-code".into()), None)
            .discard::<()>();
        let _ = settings
            .update(Action::Connect, Some(paths))
            .discard::<()>();

        settings.apply(Ok(Snapshot {
            account: Some("connected".into()),
            ..Default::default()
        }));

        assert!(settings.auth.is_none());
        assert!(settings.code.is_empty());
    }

    #[test]
    fn it_retains_selected_dropbox_folders_across_browsing_and_partial_success() {
        let mut settings = Settings::default();
        let _ = settings.update(Action::SelectFolder("/First".into(), "First".into()), None);
        settings.apply(Ok(Snapshot {
            path: "/Second".into(),
            ..Default::default()
        }));
        let _ = settings.update(
            Action::SelectFolder("/Second".into(), "Second".into()),
            None,
        );
        settings.apply(Ok(Snapshot {
            added_folders: vec!["/First".into()],
            ..settings.snapshot.clone()
        }));

        assert_eq!(
            settings.selected_folders,
            [("/Second".into(), "Second".into())]
        );
    }
}

#[cfg(test)]
#[test]
fn it_rejects_an_empty_dropbox_key_without_launching_authorization() {
    let mut settings = Settings::default();
    let task = settings.update(Action::Begin, None);

    assert_eq!(task.units(), 0);
    assert!(settings.error.is_none());
}
