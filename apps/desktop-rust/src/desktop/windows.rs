use iced::{Size, Task, window};

use super::Message;

const LIBRARY_SIZE: Size = Size::new(850.0, 570.0);
const LIBRARY_MIN: Size = Size::new(550.0, 380.0);
const MINI_SIZE: Size = Size::new(426.0, 236.0);
const MINI_MIN: Size = Size::new(370.0, 222.0);
const QUEUE_HEIGHT: f32 = 198.0;

pub struct Windows {
    library: window::Id,
    mini: Option<window::Id>,
    queue_open: bool,
    mini_bounds: Size,
}

impl Windows {
    pub fn open() -> (Self, Task<Message>) {
        let (library, open) = window::open(settings(LIBRARY_SIZE, LIBRARY_MIN));
        (
            Self {
                library,
                mini: None,
                queue_open: false,
                mini_bounds: MINI_SIZE,
            },
            open.discard(),
        )
    }

    pub fn is_mini(&self, id: window::Id) -> bool {
        self.mini == Some(id)
    }

    pub fn queue_open(&self) -> bool {
        self.queue_open
    }

    pub fn show_mini(&mut self) -> Task<Message> {
        if self.mini.is_some() {
            return Task::none();
        }
        let (id, open) = window::open(settings(self.mini_bounds, MINI_MIN));
        self.mini = Some(id);
        self.queue_open = false;
        open.map(Message::MiniOpened)
    }

    pub fn mini_opened(&self, id: window::Id) -> Task<Message> {
        if !self.is_mini(id) {
            return Task::none();
        }
        window::minimize(self.library, true)
    }

    pub fn show_library(&mut self) -> Task<Message> {
        self.mini.take().map_or_else(Task::none, |id| {
            window::minimize(self.library, false)
                .chain(window::gain_focus(self.library))
                .chain(window::close(id))
        })
    }

    pub fn toggle_queue(&mut self) -> Task<Message> {
        let Some(id) = self.mini else {
            return Task::none();
        };
        self.queue_open = !self.queue_open;
        let height = if self.queue_open { QUEUE_HEIGHT } else { 0.0 };
        window::set_min_size(
            id,
            Some(Size::new(MINI_MIN.width, MINI_MIN.height + height)),
        )
        .chain(window::resize(
            id,
            Size::new(self.mini_bounds.width, self.mini_bounds.height + height),
        ))
    }

    pub fn resized(&mut self, id: window::Id, size: Size) {
        if self.is_mini(id) {
            let height = if self.queue_open { QUEUE_HEIGHT } else { 0.0 };
            self.mini_bounds = Size::new(
                size.width.max(MINI_MIN.width),
                (size.height - height).max(MINI_MIN.height),
            );
        }
    }

    pub fn close_requested(&mut self, id: window::Id) -> Task<Message> {
        if self.is_mini(id) {
            self.show_library()
        } else {
            iced::exit()
        }
    }
}

fn settings(size: Size, min: Size) -> window::Settings {
    #[cfg(target_os = "linux")]
    let platform_specific = window::settings::PlatformSpecific {
        application_id: "muzo".into(),
        ..Default::default()
    };
    #[cfg(not(target_os = "linux"))]
    let platform_specific = window::settings::PlatformSpecific::default();

    let icon = window::icon::from_file_data(
        include_bytes!("../../../desktop/assets/icons/muzo-window.png"),
        None,
    )
    .ok();
    window::Settings {
        size,
        min_size: Some(min),
        exit_on_close_request: false,
        icon,
        platform_specific,
        ..Default::default()
    }
}

#[cfg(test)]
mod tests {
    use super::LibraryVisibility;

    #[test]
    fn it_only_minimizes_the_library_when_the_window_system_can_restore_it() {
        assert_eq!(
            LibraryVisibility::for_wayland(true),
            LibraryVisibility::KeepMapped
        );
        assert_eq!(
            LibraryVisibility::for_wayland(false),
            LibraryVisibility::MinimizeAndRestore
        );
    }
}
