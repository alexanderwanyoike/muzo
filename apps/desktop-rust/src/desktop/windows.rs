use std::time::Duration;

use iced::{Size, Task, window};

use window::raw_window_handle::RawWindowHandle;

use super::Message;

pub(super) const LIBRARY_SIZE: Size = Size::new(1200.0, 800.0);
pub(super) const LIBRARY_MIN: Size = Size::new(800.0, 600.0);
const MINI_SIZE: Size = Size::new(420.0, 560.0);
const MINI_MIN: Size = Size::new(360.0, 480.0);
const QUEUE_WIDTH: f32 = 320.0;
const QUEUE_MIN_WIDTH: f32 = 280.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum LibraryVisibility {
    KeepMapped,
    MinimizeAndRestore,
}

impl LibraryVisibility {
    fn for_wayland(wayland: bool) -> Self {
        if wayland {
            Self::KeepMapped
        } else {
            Self::MinimizeAndRestore
        }
    }
}

const RESIZE_TOLERANCE: f32 = 1.0;
const QUEUE_RESIZE_TIMEOUT: Duration = Duration::from_millis(700);

#[derive(Debug, thiserror::Error)]
pub(super) enum WindowError {
    #[error("The window could not resize. Restore its size and try again.")]
    ResizeRejected,
}

#[derive(Debug, Clone, Copy)]
struct QueueResize {
    generation: u64,
    expanded: bool,
    target: Size,
}

impl QueueResize {
    fn reached(self, size: Size) -> bool {
        (size.width - self.target.width).abs() <= RESIZE_TOLERANCE
            && (size.height - self.target.height).abs() <= RESIZE_TOLERANCE
    }
}

#[derive(Clone, Copy)]
pub(super) struct MiniPlayerLayout {
    queue_open: bool,
    resize: Option<QueueResize>,
}

impl MiniPlayerLayout {
    pub(super) fn is_resizing(self) -> bool {
        self.resize.is_some()
    }

    pub(super) fn queue_open_at(self, viewport: Size) -> bool {
        self.resize
            .filter(|resize| resize.reached(viewport))
            .map_or(self.queue_open, |resize| resize.expanded)
    }
}

pub struct Windows {
    library: window::Id,
    mini: Option<window::Id>,
    queue_open: bool,
    queue_resize: Option<QueueResize>,
    resize_generation: u64,
    mini_size: Size,
    library_visibility: LibraryVisibility,
}

impl Windows {
    pub fn open() -> (Self, Task<Message>) {
        let (library, open) = window::open(settings(LIBRARY_SIZE, LIBRARY_MIN));
        (
            Self {
                library,
                mini: None,
                queue_open: false,
                queue_resize: None,
                resize_generation: 0,
                mini_size: MINI_SIZE,
                library_visibility: LibraryVisibility::KeepMapped,
            },
            open.discard(),
        )
    }

    pub fn is_mini(&self, id: window::Id) -> bool {
        self.mini == Some(id)
    }

    pub(super) fn is_library(&self, id: window::Id) -> bool {
        self.library == id
    }

    pub(super) fn library_id(&self) -> window::Id {
        self.library
    }

    pub(super) fn mini_id(&self) -> Option<window::Id> {
        self.mini
    }

    pub(super) fn mini_view(&self) -> MiniPlayerLayout {
        MiniPlayerLayout {
            queue_open: self.queue_open,
            resize: self.queue_resize,
        }
    }

    pub fn queue_open(&self) -> bool {
        self.queue_open
    }

    pub fn show_mini(&mut self) -> Task<Message> {
        if self.mini.is_some() {
            return Task::none();
        }
        let (id, open) = window::open(settings(MINI_SIZE, MINI_MIN));
        self.mini_size = MINI_SIZE;
        self.mini = Some(id);
        self.queue_open = false;
        self.queue_resize = None;
        open.map(Message::MiniOpened)
    }

    pub fn mini_opened(&self, id: window::Id) -> Task<Message> {
        if !self.is_mini(id) {
            return Task::none();
        }
        window::run(self.library, move |handle| {
            let wayland = handle
                .window_handle()
                .is_ok_and(|handle| matches!(handle.as_raw(), RawWindowHandle::Wayland(_)));
            Message::LibraryVisibilityChecked(id, wayland)
        })
    }

    pub fn library_visibility_checked(&mut self, id: window::Id, wayland: bool) -> Task<Message> {
        if !self.is_mini(id) {
            return Task::none();
        }
        self.library_visibility = LibraryVisibility::for_wayland(wayland);
        match self.library_visibility {
            LibraryVisibility::KeepMapped => Task::none(),
            LibraryVisibility::MinimizeAndRestore => window::minimize(self.library, true),
        }
    }

    pub fn show_library(&mut self) -> Task<Message> {
        self.mini.take().map_or_else(Task::none, |id| {
            self.queue_open = false;
            self.queue_resize = None;
            let restore = match self.library_visibility {
                LibraryVisibility::KeepMapped => Task::none(),
                LibraryVisibility::MinimizeAndRestore => window::minimize(self.library, false),
            };
            restore
                .chain(window::gain_focus(self.library))
                .chain(window::close(id))
        })
    }

    pub fn queue_pending(&self) -> bool {
        self.queue_resize.is_some()
    }

    pub fn toggle_queue(&mut self) -> Task<Message> {
        let Some(id) = self.mini.filter(|_| self.queue_resize.is_none()) else {
            return Task::none();
        };
        let expanded = !self.queue_open;
        self.resize_generation += 1;
        let request = QueueResize {
            generation: self.resize_generation,
            expanded,
            target: queue_bounds(MINI_SIZE, expanded, QUEUE_WIDTH),
        };
        self.queue_resize = Some(request);

        window::set_min_size(id, Some(queue_bounds(MINI_MIN, expanded, QUEUE_MIN_WIDTH)))
            .chain(window::resize(id, request.target))
            .chain(Task::perform(
                async { tokio::time::sleep(QUEUE_RESIZE_TIMEOUT).await },
                move |()| Message::CheckQueueSize(id, request.generation),
            ))
    }

    pub fn resized(&mut self, id: window::Id, size: Size) {
        if !self.is_mini(id) {
            return;
        }
        if let Some(request) = self.queue_resize {
            if !request.reached(size) {
                return;
            }
            self.queue_open = request.expanded;
            self.queue_resize = None;
        }
        self.mini_size = size;
    }

    pub fn resize_timed_out(
        &mut self,
        id: window::Id,
        generation: u64,
        actual_size: Size,
    ) -> (Task<Message>, Result<(), WindowError>) {
        if !self.is_mini(id)
            || self
                .queue_resize
                .is_none_or(|request| request.generation != generation)
        {
            return (Task::none(), Ok(()));
        }
        self.resized(id, actual_size);
        if self.queue_resize.is_none() {
            return (Task::none(), Ok(()));
        }
        self.queue_resize = None;
        let min = queue_bounds(MINI_MIN, self.queue_open, QUEUE_MIN_WIDTH);
        let bounds = self.mini_size;
        (
            window::set_min_size(id, Some(min)).chain(window::resize(id, bounds)),
            Err(WindowError::ResizeRejected),
        )
    }

    pub fn close_requested(&mut self, id: window::Id) -> Task<Message> {
        if self.is_mini(id) {
            self.show_library()
        } else {
            iced::exit()
        }
    }
}

fn queue_bounds(base: Size, expanded: bool, queue_width: f32) -> Size {
    Size::new(
        base.width + if expanded { queue_width } else { 0.0 },
        base.height,
    )
}

fn settings(size: Size, min: Size) -> window::Settings {
    #[cfg(target_os = "linux")]
    let platform_specific = window::settings::PlatformSpecific {
        application_id: "muzo".into(),
        ..Default::default()
    };
    #[cfg(not(target_os = "linux"))]
    let platform_specific = window::settings::PlatformSpecific::default();

    let icon =
        window::icon::from_file_data(include_bytes!("../../assets/icons/muzo-window.png"), None)
            .ok();
    window::Settings {
        size,
        min_size: Some(min),
        exit_on_close_request: false,
        decorations: false,
        icon,
        platform_specific,
        ..Default::default()
    }
}

#[cfg(test)]
mod tests {
    use super::{LibraryVisibility, MINI_SIZE, Windows};
    use iced::window;

    #[test]
    fn it_keeps_the_queue_closed_until_the_native_window_has_room() {
        let (mut windows, _) = Windows::open();
        let id = window::Id::unique();
        windows.mini = Some(id);
        let _ = windows.toggle_queue();
        assert!(!windows.queue_open());
        windows.resized(id, MINI_SIZE);
        assert!(!windows.queue_open());
        windows.resized(id, iced::Size::new(740.0, 560.0));
        assert!(windows.queue_open());
    }

    #[test]
    fn it_matches_electron_compact_and_queue_dimensions() {
        let (mut windows, _) = Windows::open();
        let _ = windows.show_mini();
        let id = windows.mini_id().unwrap();
        assert_eq!(windows.mini_size, iced::Size::new(420.0, 560.0));

        let _ = windows.toggle_queue();
        let expanded = windows.queue_resize.unwrap().target;
        assert_eq!(expanded, iced::Size::new(740.0, 560.0));
        windows.resized(id, expanded);
        let _ = windows.toggle_queue();
        assert_eq!(
            windows.queue_resize.unwrap().target,
            iced::Size::new(420.0, 560.0)
        );
    }

    #[test]
    fn it_uses_the_original_size_on_mode_changes_after_manual_resizing() {
        let (mut windows, _) = Windows::open();
        let _ = windows.show_mini();
        let id = windows.mini_id().unwrap();
        windows.resized(id, iced::Size::new(405.0, 515.0));

        let _ = windows.toggle_queue();
        let expanded = windows.queue_resize.unwrap().target;
        assert_eq!(expanded, iced::Size::new(740.0, 560.0));
        windows.resized(id, expanded);
        windows.resized(id, iced::Size::new(640.0, 480.0));
        assert_eq!(windows.mini_size, iced::Size::new(640.0, 480.0));

        let _ = windows.toggle_queue();
        assert_eq!(
            windows.queue_resize.unwrap().target,
            iced::Size::new(420.0, 560.0)
        );
    }

    #[test]
    fn it_waits_for_both_target_dimensions_when_opening_the_queue() {
        let (mut windows, _) = Windows::open();
        let _ = windows.show_mini();
        let id = windows.mini_id().unwrap();
        windows.resized(id, iced::Size::new(820.0, 480.0));
        let _ = windows.toggle_queue();

        windows.resized(id, iced::Size::new(740.0, 480.0));
        assert!(windows.queue_pending());
        assert!(!windows.queue_open());
        windows.resized(id, iced::Size::new(820.0, 560.0));
        assert!(windows.queue_pending());
        windows.resized(id, iced::Size::new(740.0, 560.0));
        assert!(!windows.queue_pending());
        assert!(windows.queue_open());
    }

    #[test]
    fn it_starts_the_library_at_the_original_size() {
        assert_eq!(super::LIBRARY_SIZE, iced::Size::new(1200.0, 800.0));
        assert_eq!(super::LIBRARY_MIN, iced::Size::new(800.0, 600.0));
    }

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
