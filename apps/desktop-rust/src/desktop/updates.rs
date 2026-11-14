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
    Ready(Update, PathBuf),
    Installing,
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
    let expected = update.clone();
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
        move |result| {
            Message::UpdateReceived(match result {
                Ok(path) => Status::Ready(expected.clone(), path),
                Err(error) => Status::Error(error),
            })
        },
    )
}

pub fn launch(
    expected: &Update,
    installer: &std::path::Path,
) -> Result<(), crate::distribution::UpdateError> {
    #[cfg(target_os = "linux")]
    {
        if let Some(current) = std::env::var_os("APPIMAGE") {
            let current = std::fs::canonicalize(current)?;
            let parent = current
                .parent()
                .ok_or_else(|| std::io::Error::other("AppImage directory unavailable"))?;
            let mut update = tempfile::NamedTempFile::new_in(parent)?;
            expected.verify_and_copy(installer, update.as_file_mut())?;
            update
                .as_file()
                .set_permissions(std::fs::metadata(&current)?.permissions())?;
            update.as_file().sync_all()?;
            update.persist(&current).map_err(|error| error.error)?;
            std::process::Command::new(&current)
                .current_dir(parent)
                .stdin(std::process::Stdio::null())
                .stdout(std::process::Stdio::null())
                .stderr(std::process::Stdio::null())
                .args(std::env::args_os().skip(1))
                .env_remove("APPIMAGE")
                .env_remove("APPDIR")
                .spawn()?;
            return Ok(());
        }
    }
    expected.verify(installer)?;
    open::that_detached(installer)?;
    Ok(())
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
        Status::Ready(update, path) => button(text("Restart to install").size(11))
            .on_press(Message::InstallUpdate(update.clone(), path.clone())),
        Status::Installing => button(text("Installing update…").size(11)),
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
