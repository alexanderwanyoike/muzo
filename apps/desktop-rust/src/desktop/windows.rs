use std::time::Duration;

use iced::{Size, Task, window};

use window::raw_window_handle::{HasWindowHandle, RawWindowHandle};

use super::Message;

pub(super) const LIBRARY_SIZE: Size = Size::new(1200.0, 800.0);
pub(super) const LIBRARY_MIN: Size = Size::new(800.0, 600.0);
const MINI_SIZE: Size = Size::new(420.0, 560.0);
const MINI_MIN: Size = Size::new(360.0, 480.0);
const QUEUE_WIDTH: f32 = 320.0;
const QUEUE_MIN_WIDTH: f32 = 280.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum LibraryVisibility {
    Unknown,
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
    library_geometry: Option<(Size, bool)>,
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
                library_visibility: LibraryVisibility::Unknown,
                library_geometry: None,
            },
            open.then(|id| {
                window::run(id, move |handle| {
                    Message::LibraryVisibilityKnown(id, is_wayland(handle))
                })
            }),
        )
    }

    pub fn is_mini(&self, id: window::Id) -> bool {
        self.mini == Some(id)
    }

    pub(super) fn is_library(&self, id: window::Id) -> bool {
        self.library == id && !self.is_mini(id)
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
        self.mini_size = MINI_SIZE;
        self.queue_open = false;
        self.queue_resize = None;
        if self.library_visibility == LibraryVisibility::KeepMapped {
            return self.reuse_library_window();
        }
        let (id, open) = window::open(settings(MINI_SIZE, MINI_MIN));
        self.mini = Some(id);
        open.map(Message::MiniOpened)
    }

    pub fn library_visibility_known(&mut self, id: window::Id, wayland: bool) {
        if id == self.library && self.mini.is_none() {
            self.library_visibility = LibraryVisibility::for_wayland(wayland);
        }
    }

    fn reuse_library_window(&mut self) -> Task<Message> {
        // Wayland cannot restore a minimized window. Switch the existing window in place.
        let id = self.library;
        self.mini = Some(id);
        window::size(id).then(move |size| {
            window::is_maximized(id)
                .map(move |maximized| Message::InPlaceMiniPrepared(id, size, maximized))
        })
    }

    pub fn prepare_in_place_mini(
        &mut self,
        id: window::Id,
        size: Size,
        maximized: bool,
    ) -> Task<Message> {
        if id != self.library || !self.is_mini(id) {
            return Task::none();
        }
        self.library_geometry = Some((size, maximized));
        let resize = window::set_min_size(id, Some(MINI_MIN)).chain(window::resize(id, MINI_SIZE));
        if maximized {
            window::maximize(id, false).chain(resize)
        } else {
            resize
        }
    }

    pub fn mini_opened(&self, id: window::Id) -> Task<Message> {
        if !self.is_mini(id) {
            return Task::none();
        }
        window::run(self.library, move |handle| {
            Message::LibraryVisibilityChecked(id, is_wayland(handle))
        })
    }

    pub fn library_visibility_checked(&mut self, id: window::Id, wayland: bool) -> Task<Message> {
        if !self.is_mini(id) {
            return Task::none();
        }
        self.library_visibility = LibraryVisibility::for_wayland(wayland);
        if wayland {
            let compact = self.reuse_library_window();
            Task::batch([window::close(id), compact])
        } else {
            window::minimize(self.library, true)
        }
    }

    pub fn show_library(&mut self) -> Task<Message> {
        self.mini.take().map_or_else(Task::none, |id| {
            self.queue_open = false;
            self.queue_resize = None;
            if id == self.library {
                return self.library_geometry.take().map_or_else(
                    Task::none,
                    |(size, maximized)| {
                        let resize = window::set_min_size(id, Some(LIBRARY_MIN))
                            .chain(window::resize(id, size));
                        if maximized {
                            resize.chain(window::maximize(id, true))
                        } else {
                            resize
                        }
                    },
                );
            }
            let restore = match self.library_visibility {
                LibraryVisibility::Unknown | LibraryVisibility::KeepMapped => Task::none(),
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

fn is_wayland(handle: &dyn HasWindowHandle) -> bool {
    handle
        .window_handle()
        .is_ok_and(|handle| matches!(handle.as_raw(), RawWindowHandle::Wayland(_)))
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
    fn it_uses_one_wayland_window_for_the_library_and_mini_player() {
        let (mut windows, _) = Windows::open();
        let library = windows.library_id();
        let _ = windows.show_mini();
        let initial_mini = windows.mini_id().unwrap();

        let _ = windows.library_visibility_checked(initial_mini, true);

        assert_eq!(
            windows.mini_id(),
            Some(library),
            "Wayland compact mode should reuse the library window"
        );
        assert!(!windows.is_library(library));
        let _ = windows.show_library();
        assert!(windows.is_library(library));
        assert!(windows.mini_id().is_none());
    }

    #[test]
    fn it_does_not_request_an_unmaximize_configure_for_a_floating_wayland_window() {
        use iced::futures::{FutureExt, StreamExt};
        use iced_runtime::{Action, task};

        let (mut windows, _) = Windows::open();
        let id = windows.library_id();
        windows.library_visibility_known(id, true);
        let _ = windows.show_mini();

        let compact = windows.prepare_in_place_mini(id, super::LIBRARY_SIZE, false);
        let mut actions = task::into_stream(compact).unwrap();
        assert!(matches!(
            actions.next().now_or_never().flatten(),
            Some(Action::Window(iced::window::Action::SetMinSize(window, _))) if window == id
        ));
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
