use std::path::PathBuf;

use iced::{
    Element, Task,
    widget::{button, row, text},
};

use crate::distribution::{Target, Update, latest};

use super::Message;

#[derive(Debug, Clone, Default)]
pub enum Status {
    #[default]
    Idle,
    Checking,
    Available(Update),
    Downloading,
    Ready(PathBuf),
    Error(String),
}

pub fn check() -> Task<Message> {
    let Some(target) = Target::current() else {
        return Task::none();
    };
    Task::perform(
        async move {
            tokio::task::spawn_blocking(move || {
                latest(env!("CARGO_PKG_VERSION"), target).map_err(|error| error.to_string())
            })
            .await
            .map_err(|error| format!("Could not check for updates: {error}"))?
        },
        |result| {
            Message::UpdateReceived(match result {
                Ok(Some(update)) => Status::Available(update),
                Ok(None) => Status::Idle,
                Err(error) => Status::Error(error),
            })
        },
    )
}

pub fn download(update: Update, directory: PathBuf) -> Task<Message> {
    Task::perform(
        async move {
            tokio::task::spawn_blocking(move || {
                update
                    .download(&directory)
                    .map_err(|error| error.to_string())
            })
            .await
            .map_err(|error| format!("Could not download the update: {error}"))?
        },
        |result| {
            Message::UpdateReceived(match result {
                Ok(path) => Status::Ready(path),
                Err(error) => Status::Error(error),
            })
        },
    )
}

pub fn launch(installer: &std::path::Path) -> std::io::Result<()> {
    #[cfg(target_os = "linux")]
    {
        use std::os::unix::fs::PermissionsExt;
        if let Some(current) = std::env::var_os("APPIMAGE") {
            let current = std::fs::canonicalize(current)?;
            let parent = current
                .parent()
                .ok_or_else(|| std::io::Error::other("AppImage directory unavailable"))?;
            let mut update = tempfile::NamedTempFile::new_in(parent)?;
            std::io::copy(&mut std::fs::File::open(installer)?, update.as_file_mut())?;
            update
                .as_file()
                .set_permissions(std::fs::Permissions::from_mode(0o755))?;
            update.as_file().sync_all()?;
            update.persist(&current).map_err(|error| error.error)?;
            std::process::Command::new(current)
                .args(std::env::args_os().skip(1))
                .env_remove("APPIMAGE")
                .env_remove("APPDIR")
                .spawn()?;
            return Ok(());
        }
    }
    open::that_detached(installer)
}

pub fn view(status: &Status) -> Element<'_, Message> {
    let summary = match status {
        Status::Idle => button(text(format!("Muzo {}", env!("CARGO_PKG_VERSION"))).size(11))
            .on_press(Message::CheckUpdate),
        Status::Checking => button(text("Checking for updates…").size(11)),
        Status::Available(update) => {
            button(text(format!("Muzo {} available", update.version)).size(11))
                .on_press(Message::DownloadUpdate(update.clone()))
        }
        Status::Downloading => button(text("Downloading update…").size(11)),
        Status::Ready(path) => button(text("Restart to install").size(11))
            .on_press(Message::InstallUpdate(path.clone())),
        Status::Error(error) => {
            return row![
                text(error).size(11),
                button("Retry").on_press(Message::CheckUpdate)
            ]
            .spacing(5)
            .into();
        }
    };
    summary.into()
}
