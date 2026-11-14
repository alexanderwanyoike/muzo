mod view;
mod windows;

use std::{path::PathBuf, time::Duration};

use iced::{Element, Subscription, Task, Theme, widget::image, window};
use rodio::{DeviceSinkBuilder, MixerDeviceSink};

use crate::{
    local_track::read_track,
    playback::Playback,
    session::{Session, SessionError},
    track::Track,
};

use windows::Windows;

#[derive(Debug, Clone)]
pub enum Message {
    OpenFiles,
    FilesPicked(Vec<PathBuf>),
    FilesRead(Result<Vec<Track>, String>),
    Select(usize),
    TogglePlayback,
    Previous,
    Next,
    Seek(f32),
    SetVolume(f32),
    Tick,
    ShowMini,
    MiniOpened(window::Id),
    LibraryVisibilityChecked(window::Id, bool),
    ShowLibrary,
    ToggleQueue,
    CloseRequested(window::Id),
    MiniResized(window::Id, iced::Size),
}

pub struct Desktop {
    windows: Windows,
    session: Option<Session>,
    output: Option<MixerDeviceSink>,
    error: Option<String>,
    artwork: Option<image::Handle>,
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
        let files = std::env::args_os().skip(1).map(PathBuf::from).collect();
        (desktop, open.chain(Task::done(Message::FilesPicked(files))))
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
                artwork: None,
            },
            open,
        )
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::OpenFiles => Task::perform(
                async {
                    rfd::AsyncFileDialog::new()
                        .add_filter("Audio", &["mp3", "flac", "m4a", "ogg", "opus", "wav"])
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
                    tokio::task::spawn_blocking(move || paths.into_iter().map(read_track).collect())
                        .await
                        .map_err(|error| format!("Could not read the selected tracks: {error}"))
                },
                Message::FilesRead,
            ),
            Message::FilesRead(Ok(tracks)) => {
                if let Some(session) = &mut self.session
                    && !tracks.is_empty()
                {
                    let first = session.tracks().len();
                    session.extend(tracks);
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
            Message::Select(index) => self.transport(|session| session.select(index)),
            Message::TogglePlayback => self.transport(Session::toggle),
            Message::Previous => self.transport(Session::previous_track),
            Message::Next => self.transport(Session::next_track),
            Message::Seek(seconds) => {
                self.transport(|session| session.seek(Duration::from_secs_f32(seconds)))
            }
            Message::SetVolume(value) => self.transport(|session| {
                session.set_volume(value);
                Ok(())
            }),
            Message::Tick => {
                self.refresh_transport(Session::advance_if_finished, false);
                Task::none()
            }
            Message::ShowMini => self.windows.show_mini(),
            Message::MiniOpened(id) => self.windows.mini_opened(id),
            Message::LibraryVisibilityChecked(id, wayland) => {
                self.windows.library_visibility_checked(id, wayland)
            }
            Message::ShowLibrary => self.windows.show_library(),
            Message::ToggleQueue => self.windows.toggle_queue(),
            Message::CloseRequested(id) => self.windows.close_requested(id),
            Message::MiniResized(id, size) => {
                self.windows.resized(id, size);
                Task::none()
            }
        }
    }

    fn transport(
        &mut self,
        operation: impl FnOnce(&mut Session) -> Result<(), SessionError>,
    ) -> Task<Message> {
        self.refresh_transport(operation, true);
        Task::none()
    }

    fn refresh_transport(
        &mut self,
        operation: impl FnOnce(&mut Session) -> Result<(), SessionError>,
        clear_on_success: bool,
    ) {
        if let Some(session) = &mut self.session {
            let current = session.current_index();
            match operation(session) {
                Ok(()) => {
                    if clear_on_success || current != session.current_index() {
                        self.error = None;
                    }
                }
                Err(error) => self.error = Some(error.to_string()),
            }
            if current != session.current_index() {
                self.artwork = session
                    .current()
                    .and_then(Track::artwork)
                    .map(|encoded| image::Handle::from_bytes(encoded.to_vec()));
            }
        }
    }

    pub fn subscription(&self) -> Subscription<Message> {
        Subscription::batch([
            window::close_requests().map(Message::CloseRequested),
            window::resize_events().map(|(id, size)| Message::MiniResized(id, size)),
            if self.session.as_ref().is_some_and(Session::is_playing) {
                iced::time::every(Duration::from_millis(100)).map(|_| Message::Tick)
            } else {
                Subscription::none()
            },
            iced::event::listen_with(|event, status, _| {
                if status == iced::event::Status::Captured {
                    return None;
                }
                if let iced::Event::Keyboard(iced::keyboard::Event::KeyPressed {
                    key: iced::keyboard::Key::Named(iced::keyboard::key::Named::Space),
                    ..
                }) = event
                {
                    return Some(Message::TogglePlayback);
                }
                None
            }),
        ])
    }

    pub fn title(&self, id: window::Id) -> String {
        if self.windows.is_mini(id) {
            "Muzo · Mini player"
        } else {
            "Muzo"
        }
        .into()
    }

    pub fn view(&self, id: window::Id) -> Element<'_, Message> {
        if self.windows.is_mini(id) {
            view::mini(
                self.session.as_ref(),
                self.windows.queue_open(),
                self.artwork.as_ref(),
                self.error.as_deref(),
            )
        } else {
            view::library(
                self.session.as_ref(),
                self.artwork.as_ref(),
                self.error.as_deref(),
            )
        }
    }

    pub fn theme(&self, _id: window::Id) -> Theme {
        view::theme()
    }

    #[cfg(test)]
    pub fn now_playing(&self) -> Option<(usize, Duration)> {
        let session = self.session.as_ref()?;
        Some((session.current_index()?, session.observe().0))
    }
}
