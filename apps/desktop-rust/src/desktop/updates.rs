use std::path::PathBuf;

use iced::{
    Element, Task, border,
    widget::{button, row, text, tooltip},
};

use crate::{
    appearance::Surfaces,
    distribution::{Target, Update, latest},
};

use super::Message;

#[derive(Debug, Clone, Default)]
pub enum Status {
    #[default]
    Idle,
    Checking,
    Current,
    Available(Update),
    Downloading(u8),
    Ready(Update, PathBuf),
    Confirming(Update, PathBuf),
    Installing,
    Error(String),
}

impl Status {
    pub(super) fn from_check(result: Result<Option<Update>, String>) -> Self {
        match result {
            Ok(Some(update)) => Self::Available(update),
            Ok(None) => Self::Current,
            Err(error) => Self::Error(error),
        }
    }

    pub(super) fn can_check(&self) -> bool {
        matches!(self, Self::Idle | Self::Current | Self::Error(_))
    }

    pub(super) fn notice_label(&self) -> String {
        match self {
            Self::Idle => "Check for updates".into(),
            Self::Checking => "Checking for updates…".into(),
            Self::Current => "Up to date".into(),
            Self::Available(update) => format!("Update available to {}", update.version),
            Self::Downloading(percent) => format!("Downloading update… {percent}%"),
            Self::Ready(update, _) => format!("Restart to install v{}", update.version),
            Self::Confirming(update, _) => format!("Restart to install {}?", update.version),
            Self::Installing => "Installing update…".into(),
            Self::Error(_) => "Update failed. Retry".into(),
        }
    }

    pub(super) fn accepts(&self, incoming: &Self) -> bool {
        match (self, incoming) {
            (Self::Checking, Self::Current | Self::Available(_) | Self::Error(_)) => true,
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
        |result| Message::UpdateReceived(Status::from_check(result)),
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

pub fn view(status: &Status, surfaces: Surfaces) -> Element<'_, Message> {
    let label = status.notice_label();
    let (action, hint) = match status {
        Status::Idle | Status::Current => (
            installed_build().then_some(Message::CheckUpdate),
            "Check for a new Muzo version",
        ),
        Status::Available(update) => (
            Some(Message::DownloadUpdate(update.clone())),
            if manual_download() {
                "Download installer"
            } else {
                "Download update"
            },
        ),
        Status::Ready(update, path) => (
            Some(Message::InstallUpdate(update.clone(), path.clone())),
            "Your update is ready",
        ),
        Status::Error(error) => (Some(Message::CheckUpdate), error.as_str()),
        Status::Confirming(_, _) => {
            return row![
                text(label).size(11),
                pill("Later", Some(Message::CancelInstallUpdate), surfaces),
                pill("Restart", Some(Message::ConfirmInstallUpdate), surfaces),
            ]
            .spacing(5)
            .align_y(iced::Alignment::Center)
            .into();
        }
        _ => return pill(label, None, surfaces).into(),
    };
    tooltip(
        pill(label, action, surfaces),
        text(hint).size(11),
        tooltip::Position::Top,
    )
    .style(super::view::panel)
    .into()
}

fn pill<'a>(
    label: impl Into<String>,
    action: Option<Message>,
    surfaces: Surfaces,
) -> button::Button<'a, Message> {
    button(text(label.into()).size(10))
        .height(24)
        .padding([2, 10])
        .style(move |theme: &iced::Theme, status| {
            let hovered = matches!(status, button::Status::Hovered | button::Status::Pressed);
            button::Style {
                background: Some(
                    if hovered {
                        theme.extended_palette().background.strong.color
                    } else {
                        surfaces.raised
                    }
                    .into(),
                ),
                text_color: if status == button::Status::Disabled {
                    surfaces.muted
                } else {
                    theme.palette().text
                },
                border: border::rounded(12).width(1).color(surfaces.line),
                ..button::Style::default()
            }
        })
        .on_press_maybe(action)
}

#[cfg(test)]
mod tests {
    use iced::{
        Font, Pixels, Renderer, Size,
        advanced::{layout, widget::Tree},
    };

    use super::*;

    #[test]
    fn it_reserves_footer_space_for_the_update_failure_reason() {
        let surfaces = crate::appearance::Appearance::default().surfaces();
        let renderer = Renderer::new(Font::DEFAULT, Pixels(16.0));
        let limits = layout::Limits::new(Size::ZERO, Size::new(700.0, 34.0));
        let width = |reason| {
            let status = Status::Error(reason);
            let mut element = view(&status, surfaces);
            let mut tree = Tree::new(&element);
            element
                .as_widget_mut()
                .layout(&mut tree, &renderer, &limits)
                .size()
                .width
        };
        assert!(width("Connection closed during the download".into()) > width("Offline".into()));
    }
}
