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
    Downloading(u8),
    Ready(Update, PathBuf),
    Confirming(Update, PathBuf),
    Installing,
    Error(String),
}

impl Status {
    pub(super) fn accepts(&self, incoming: &Self) -> bool {
        match (self, incoming) {
            (Self::Checking, Self::Idle | Self::Available(_) | Self::Error(_)) => true,
            (Self::Downloading(_), Self::Ready(_, _) | Self::Error(_)) => true,
            (Self::Downloading(previous), Self::Downloading(current)) => current >= previous,
            _ => false,
        }
    }
}

pub(super) fn installed_build() -> bool {
    (cfg!(target_os = "linux") && std::env::var_os("APPIMAGE").is_some())
        || (cfg!(any(target_os = "windows", target_os = "macos")) && !cfg!(debug_assertions))
}

fn manual_download() -> bool {
    cfg!(target_os = "macos") || (cfg!(target_os = "linux") && !installed_build())
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
    Task::run(
        iced::stream::channel(
            8,
            move |output: iced::futures::channel::mpsc::Sender<Status>| async move {
                use iced::futures::SinkExt;

                let mut output = output;
                let worker_output = output.clone();
                let result = tokio::task::spawn_blocking(move || {
                    let mut output = worker_output;
                    let result = update.download(&directory, |percentage| {
                        let _ = output.try_send(Status::Downloading(percentage));
                    });
                    match result {
                        Ok(path) => Status::Ready(update, path),
                        Err(error) => Status::Error(error.to_string()),
                    }
                })
                .await
                .unwrap_or_else(|error| {
                    Status::Error(format!("Could not download the update: {error}"))
                });
                let _ = output.send(result).await;
            },
        ),
        Message::UpdateReceived,
    )
}

pub fn launch(
    expected: &Update,
    installer: &std::path::Path,
) -> Result<(), crate::distribution::UpdateError> {
    #[cfg(target_os = "linux")]
    {
        if let Some(current) = std::env::var_os("APPIMAGE") {
            let original_directory = std::env::var_os("OWD")
                .map(PathBuf::from)
                .filter(|path| path.is_absolute())
                .map_or_else(std::env::current_dir, Ok)?;
            let arguments = crate::distribution::appimage::relaunch_arguments(
                &original_directory,
                std::env::args_os().skip(1),
            );
            return crate::distribution::appimage::replace_and_launch(
                expected,
                installer,
                std::path::Path::new(&current),
                arguments,
            );
        }
    }
    expected.verify(installer)?;
    open::that_detached(installer)?;
    Ok(())
}

pub fn view(status: &Status) -> Element<'_, Message> {
    let summary = match status {
        Status::Idle => button(text(format!("Muzo {}", env!("CARGO_PKG_VERSION"))).size(11))
            .on_press_maybe(installed_build().then_some(Message::CheckUpdate)),
        Status::Checking => button(text("Checking for updates…").size(11)),
        Status::Available(update) => {
            let label = if manual_download() {
                format!("Download installer for {}", update.version)
            } else {
                format!("Update available to {}", update.version)
            };
            button(text(label).size(11)).on_press(Message::DownloadUpdate(update.clone()))
        }
        Status::Downloading(percent) => {
            button(text(format!("Downloading update… {percent}%")).size(11))
        }
        Status::Ready(update, path) => button(text("Restart to install").size(11))
            .on_press(Message::InstallUpdate(update.clone(), path.clone())),
        Status::Confirming(update, _) => {
            return row![
                text(format!("Restart to install {}?", update.version)).size(11),
                button("Later").on_press(Message::CancelInstallUpdate),
                button("Restart").on_press(Message::ConfirmInstallUpdate),
            ]
            .spacing(5)
            .align_y(iced::Alignment::Center)
            .into();
        }
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
