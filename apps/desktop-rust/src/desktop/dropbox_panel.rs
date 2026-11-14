use std::{path::PathBuf, sync::Arc, time::Duration};

use iced::{
    Task,
    widget::{button, column, row, scrollable, text, text_input},
};

use crate::{
    catalogue::{Catalogue, CatalogueError, Library},
    dropbox::{Authorization, Client, Credentials, DropboxError, Entry, cache},
};

use super::{Message, library_sources::CataloguePaths};

#[derive(Debug, Clone)]
pub enum Action {
    Toggle,
    Key(String),
    Code(String),
    Begin,
    Connect,
    Disconnect,
    Browse(String),
    Add(String, String),
    Interval(String),
    SaveInterval,
    Sync,
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
}

pub struct Settings {
    pub visible: bool,
    pub busy: bool,
    pub error: Option<String>,
    pub app_key: String,
    pub code: String,
    pub interval: String,
    pub snapshot: Snapshot,
    auth: Option<Arc<Authorization>>,
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
            visible: false,
            busy: false,
            error: None,
            app_key: String::new(),
            code: String::new(),
            interval: "5".into(),
            snapshot: Snapshot::default(),
            auth: None,
            next_sync: std::time::Instant::now(),
        }
    }
}

impl Settings {
    pub fn update(&mut self, action: Action, paths: Option<CataloguePaths>) -> Task<Message> {
        match action {
            Action::Toggle => {
                self.visible = !self.visible;
                return Task::none();
            }
            Action::Key(value) => {
                self.app_key = value;
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
        if matches!(action, Action::Begin) {
            return match Authorization::new(&self.app_key).and_then(|authorization| {
                let url = authorization.url()?;
                self.auth = Some(Arc::new(authorization));
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
        let auth = self.auth.clone();
        let code = std::mem::take(&mut self.code);
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

    pub fn apply(&mut self, result: Result<Snapshot, String>) {
        self.busy = false;
        match result {
            Ok(snapshot) => {
                if self.app_key.is_empty() {
                    self.app_key = snapshot.app_key.clone();
                }
                self.interval = snapshot.interval.to_string();
                self.next_sync = std::time::Instant::now()
                    + Duration::from_secs(u64::from(snapshot.interval.max(1)) * 60);
                self.snapshot = snapshot;
            }
            Err(error) => {
                self.error = Some(error);
            }
        }
    }

    pub fn view(&self) -> iced::widget::Column<'_, Message> {
        let mut inner = column![text("Dropbox").size(19)];
        let msg = Message::Dropbox;
        if let Some(account) = &self.snapshot.account {
            inner = inner.push(
                text(if self.snapshot.reconnect {
                    "Reconnect Dropbox"
                } else {
                    account
                })
                .size(13),
            );
        }
        if self.snapshot.account.is_none() || self.snapshot.reconnect || self.auth.is_some() {
            inner = inner.push(
                row![
                    text_input("Dropbox app key", &self.app_key)
                        .on_input(move |value| msg(Action::Key(value))),
                    button("Connect Dropbox")
                        .on_press_maybe((!self.busy).then(|| msg(Action::Begin))),
                ]
                .spacing(7),
            );
        }
        if self.auth.is_some() {
            inner = inner.push(
                row![
                    text_input("Authorization code", &self.code)
                        .secure(true)
                        .on_input(move |value| msg(Action::Code(value))),
                    button("Connect").on_press_maybe((!self.busy).then(|| msg(Action::Connect))),
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
                        button("Add this folder").on_press_maybe(
                            (!self.busy).then(|| msg(Action::Add(path.clone(), folder_name(path))))
                        ),
                    ]
                    .spacing(7),
                );
                let rows = self.snapshot.folders.iter().map(|(path, name)| {
                    row![
                        button(text(name)).on_press_maybe(
                            (!self.busy).then(|| msg(Action::Browse(path.clone())))
                        ),
                        button("Add").on_press_maybe(
                            (!self.busy).then(|| msg(Action::Add(path.clone(), name.clone())))
                        ),
                    ]
                    .spacing(7)
                    .into()
                });
                inner = inner.push(scrollable(column(rows).spacing(4)).height(iced::Fill));
            }
        }
        if self.busy {
            inner = inner.push(text("Connecting…").size(12));
        }
        if let Some(error) = &self.error {
            inner = inner.push(text(error).size(12));
        }
        inner.spacing(10)
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

    let current = catalogue.latest_dropbox()?;
    snapshot.disconnected_libraries.clear();
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
    if matches!(action, Action::Disconnect) {
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
        Action::Add(path, name) => {
            let token = match refresh_current() {
                Ok(token) => token,
                Err(DropboxError::Reconnect) => {
                    snapshot.reconnect = true;
                    return Ok(snapshot);
                }
                Err(error) => return Err(error.into()),
            };
            let entries = client.list(&token.access_token, &path, true)?;
            let library = catalogue.add_dropbox(&account, &path, &name)?;
            catalogue.sync_dropbox(&library.id, &entries)?;
        }
        Action::Sync | Action::Connect => {
            for library in catalogue
                .libraries()?
                .into_iter()
                .filter(|library| library.kind == "dropbox")
            {
                if let Some(owner) = catalogue.dropbox_account_for(&library.id)? {
                    let Some(key) = catalogue.dropbox_client(&owner)? else {
                        continue;
                    };
                    let Some(refresh) = Credentials::read_or_migrate(&catalogue, &owner)? else {
                        continue;
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
