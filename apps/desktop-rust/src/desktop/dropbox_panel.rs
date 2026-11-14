use std::{path::PathBuf, sync::Arc, time::Duration};

use iced::{
    Task,
    widget::{button, checkbox, column, row, scrollable, text, text_input},
};

use crate::{
    appearance::Surfaces,
    catalogue::{Catalogue, CatalogueError, Library},
    dropbox::{Authorization, Client, Credentials, DropboxError, Entry, cache},
};

use super::{
    Message,
    library_sources::CataloguePaths,
    view::{settings_button, settings_input},
};

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
        !self.busy && self.snapshot.account.is_some() && !self.snapshot.reconnect
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

    pub fn view(&self, surfaces: Surfaces) -> iced::widget::Column<'_, Message> {
        let mut inner = column![].spacing(14);
        let msg = Message::Dropbox;
        if self.snapshot.account.is_some() {
            inner = inner.push(
                text(if self.snapshot.reconnect {
                    "Reconnect Dropbox"
                } else {
                    "Connected to Dropbox."
                })
                .size(13),
            );
        }
        if self.snapshot.account.is_none() || self.snapshot.reconnect || self.auth.is_some() {
            inner = inner
                .push(
                    column![
                        text("Dropbox app key").size(11).color(surfaces.muted),
                        settings_input(&self.app_key, surfaces).on_input_maybe(
                            (!self.busy && self.auth.is_none())
                                .then_some(move |value| msg(Action::Key(value)))
                        ),
                    ]
                    .spacing(6),
                )
                .push(
                    settings_button(
                        if self.auth.is_some() {
                            "Restart authorization"
                        } else {
                            "Connect Dropbox"
                        },
                        surfaces,
                    )
                    .on_press_maybe(
                        (!self.busy && !self.app_key.trim().is_empty())
                            .then_some(msg(Action::Begin)),
                    ),
                );
        }
        if self.auth.is_some() {
            inner = inner.push(
                column![
                    text("Approve access in your browser, then paste the authorization code here.")
                        .size(11),
                    text("Authorization code").size(11).color(surfaces.muted),
                    settings_input(&self.code, surfaces)
                        .secure(true)
                        .on_input_maybe(
                            (!self.busy).then_some(move |value| msg(Action::Code(value)))
                        ),
                    settings_button("Finish connecting", surfaces).on_press_maybe(
                        (!self.busy && !self.code.trim().is_empty())
                            .then_some(msg(Action::Connect))
                    ),
                    settings_button("Cancel connection", surfaces)
                        .on_press_maybe((!self.busy).then_some(msg(Action::CancelAuthorization))),
                ]
                .spacing(7),
            );
        }
        if self.snapshot.account.is_some() && !self.snapshot.reconnect {
            inner = inner.push(
                row![
                    button("Browse Dropbox folders")
                        .on_press_maybe((!self.busy).then(|| msg(Action::Browse("/".into())))),
                    button("Refresh").on_press_maybe((!self.busy).then(|| msg(Action::Sync))),
                    button("Disconnect")
                        .on_press_maybe((!self.busy).then(|| msg(Action::Disconnect))),
                ]
                .spacing(7),
            );
            inner = inner.push(
                row![
                    text_input("Sync interval in minutes", &self.interval)
                        .on_input(move |value| msg(Action::Interval(value)))
                        .width(190),
                    button("Save interval")
                        .on_press_maybe((!self.busy).then(|| msg(Action::SaveInterval))),
                ]
                .spacing(7),
            );
            if !self.snapshot.path.is_empty() {
                let path = &self.snapshot.path;
                let up = path
                    .rsplit_once('/')
                    .map(|(up, _)| if up.is_empty() { "/" } else { up })
                    .unwrap_or("/");
                inner = inner.push(
                    row![
                        button("Up").on_press(msg(Action::Browse(up.into()))),
                        text(path).size(13),
                        self.folder_checkbox(path, &folder_name(path)),
                    ]
                    .spacing(7),
                );
                let rows = self.snapshot.folders.iter().map(|(path, name)| {
                    row![
                        self.folder_checkbox(path, name),
                        button(text(name)).on_press_maybe(
                            (!self.busy).then(|| msg(Action::Browse(path.clone())))
                        ),
                    ]
                    .spacing(7)
                    .into()
                });
                inner = inner.push(scrollable(column(rows).spacing(4)).height(iced::Fill));
                if !self.selected_folders.is_empty() {
                    for (path, name) in &self.selected_folders {
                        inner = inner.push(
                            row![
                                text(path).size(12),
                                button("Remove").on_press_maybe((!self.busy).then(|| {
                                    msg(Action::SelectFolder(path.clone(), name.clone()))
                                }))
                            ]
                            .spacing(7),
                        );
                    }
                    inner = inner.push(
                        button("Add selected folders")
                            .on_press_maybe((!self.busy).then_some(msg(Action::AddSelected))),
                    );
                }
            }
        }
        if self.busy {
            inner = inner.push(text("Connecting…").size(12));
        }
        if let Some(error) = &self.error {
            inner = inner.push(text(error).size(12));
        }
        inner.width(iced::Fill)
    }

    fn folder_checkbox(&self, path: &str, name: &str) -> iced::widget::Checkbox<'_, Message> {
        let added = self
            .snapshot
            .added_paths
            .iter()
            .any(|added| added.eq_ignore_ascii_case(path));
        let selected = self
            .selected_folders
            .iter()
            .any(|(item, _)| item.eq_ignore_ascii_case(path));
        let label = if added {
            format!("{name} (added)")
        } else {
            format!("Select {name}")
        };
        let action = Action::SelectFolder(path.to_owned(), name.to_owned());
        checkbox(added || selected).label(label).on_toggle_maybe(
            (!self.busy && !added).then_some(move |_| Message::Dropbox(action.clone())),
        )
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
        let minutes = interval
            .trim()
            .parse::<u16>()
            .ok()
            .filter(|value| (1..=60).contains(value))
            .ok_or(Error::InvalidInterval)?;
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
        Action::Sync | Action::SyncLibrary(_) => {
            let selected = match &action {
                Action::SyncLibrary(id) => Some(id.as_str()),
                _ => None,
            };
            for library in catalogue
                .libraries()?
                .into_iter()
                .filter(|library| library.kind == "dropbox")
                .filter(|library| selected.is_none_or(|id| library.id == id))
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

pub fn remote_audio(paths: &CataloguePaths, library: &str, path: &str) -> Result<PathBuf, Error> {
    let catalogue: Catalogue = paths.open()?;
    let account = catalogue
        .dropbox_account_for(library)?
        .ok_or(DropboxError::Reconnect)?;
    let client_id = catalogue
        .dropbox_client(&account)?
        .ok_or(DropboxError::Reconnect)?;
    let refresh =
        Credentials::read_or_migrate(&catalogue, &account)?.ok_or(DropboxError::Reconnect)?;
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

        let result = run(
            &paths,
            Action::SyncLibrary(library.id),
            None,
            "",
            "5",
            Snapshot::default(),
        );

        assert!(matches!(
            result,
            Err(Error::Dropbox(DropboxError::Reconnect))
        ));
    }

    #[test]
    fn it_leaves_the_background_schedule_and_pending_drafts_alone_on_source_refresh() {
        let directory = tempfile::tempdir().unwrap();
        let paths = CataloguePaths::in_directory(directory.path());
        let mut settings = Settings::default();
        settings.code = "pending-code".into();
        settings.interval = "30".into();
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
