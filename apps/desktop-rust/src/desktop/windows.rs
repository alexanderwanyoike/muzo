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
    ReplaceSurface,
    MinimizeAndRestore,
}

impl LibraryVisibility {
    fn for_wayland(wayland: bool) -> Self {
        if wayland {
            Self::ReplaceSurface
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
        if self.library_visibility == LibraryVisibility::ReplaceSurface {
            return self.prepare_wayland_window();
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

    fn prepare_wayland_window(&mut self) -> Task<Message> {
        let id = self.library;
        self.mini = Some(id);
        window::size(id).then(move |size| {
            window::is_maximized(id)
                .map(move |maximized| Message::WaylandMiniPrepared(id, size, maximized))
        })
    }

    pub fn open_wayland_mini(
        &mut self,
        id: window::Id,
        size: Size,
        maximized: bool,
    ) -> Task<Message> {
        if id != self.library || !self.is_mini(id) {
            return Task::none();
        }
        self.library_geometry = Some((size, maximized));
        self.replace_mini_surface(id, false)
    }

    fn replace_mini_surface(&mut self, previous: window::Id, expanded: bool) -> Task<Message> {
        // Iced 0.14 waits for an event that Winit omits after a synchronous Wayland resize.
        let size = queue_bounds(MINI_SIZE, expanded, QUEUE_WIDTH);
        let min = queue_bounds(MINI_MIN, expanded, QUEUE_MIN_WIDTH);
        let (id, open) = window::open(settings(size, min));
        self.mini = Some(id);
        self.queue_open = expanded;
        self.mini_size = size;
        window::close(previous).chain(open.map(Message::WaylandMiniOpened))
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
            let compact = self.prepare_wayland_window();
            window::close(id).chain(compact)
        } else {
            window::minimize(self.library, true)
        }
    }

    pub fn show_library(&mut self) -> Task<Message> {
        self.mini.take().map_or_else(Task::none, |id| {
            self.queue_open = false;
            self.queue_resize = None;
            if self.library_visibility == LibraryVisibility::ReplaceSurface {
                let Some((size, maximized)) = self.library_geometry.take() else {
                    return Task::none();
                };
                let mut settings = settings(size, LIBRARY_MIN);
                settings.maximized = maximized;
                let (library, open) = window::open(settings);
                self.library = library;
                return window::close(id).chain(open.discard());
            }
            let restore = match self.library_visibility {
                LibraryVisibility::Unknown | LibraryVisibility::ReplaceSurface => Task::none(),
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
        if self.library_visibility == LibraryVisibility::ReplaceSurface {
            return if id == self.library {
                Task::none()
            } else {
                self.replace_mini_surface(id, expanded)
            };
        }
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
    fn it_replaces_the_visible_wayland_surface_between_library_and_mini_player() {
        let (mut windows, _) = Windows::open();
        let library = windows.library_id();
        let _ = windows.show_mini();
        let initial_mini = windows.mini_id().unwrap();

        let _ = windows.library_visibility_checked(initial_mini, true);
        let _ = windows.open_wayland_mini(library, super::LIBRARY_SIZE, false);

        assert_ne!(windows.mini_id(), Some(library));
        let _ = windows.show_library();
        assert_ne!(windows.library_id(), library);
        assert!(windows.is_library(windows.library_id()));
        assert!(windows.mini_id().is_none());
    }

    #[test]
    fn it_restores_the_original_wayland_library_size_and_maximized_state() {
        use iced::futures::{FutureExt, StreamExt};
        use iced_runtime::{Action, task};

        let (mut windows, _) = Windows::open();
        let id = windows.library_id();
        windows.library_visibility_known(id, true);
        let _ = windows.show_mini();
        let size = iced::Size::new(1050.0, 750.0);
        let _ = windows.open_wayland_mini(id, size, true);
        let mini = windows.mini_id().unwrap();

        let mut actions = task::into_stream(windows.show_library()).unwrap();
        assert!(matches!(
            actions.next().now_or_never().flatten(),
            Some(Action::Window(iced::window::Action::Close(window))) if window == mini
        ));
        assert!(matches!(
            actions.next().now_or_never().flatten(),
            Some(Action::Window(iced::window::Action::Open(window, settings, _)))
                if window == windows.library_id() && settings.size == size && settings.maximized
        ));
    }

    #[test]
    fn it_opens_wayland_queue_with_a_new_surface_at_the_target_size() {
        use iced::futures::{FutureExt, StreamExt};
        use iced_runtime::{Action, task};

        let (mut windows, _) = Windows::open();
        let library = windows.library_id();
        windows.library_visibility_known(library, true);
        let _ = windows.show_mini();
        let _ = windows.open_wayland_mini(library, super::LIBRARY_SIZE, false);
        let compact = windows.mini_id().unwrap();

        let expanded = windows.toggle_queue();
        let id = windows.mini_id().unwrap();
        let mut actions = task::into_stream(expanded).unwrap();
        assert!(matches!(
            actions.next().now_or_never().flatten(),
            Some(Action::Window(iced::window::Action::Close(window))) if window == compact
        ));
        assert!(matches!(
            actions.next().now_or_never().flatten(),
            Some(Action::Window(iced::window::Action::Open(window, settings, _)))
                if window == id && window != compact && settings.size == iced::Size::new(740.0, 560.0)
        ));
        assert!(windows.queue_open());
    }

    #[test]
    fn it_only_minimizes_the_library_when_the_window_system_can_restore_it() {
        assert_eq!(
            LibraryVisibility::for_wayland(true),
            LibraryVisibility::ReplaceSurface
        );
        assert_eq!(
            LibraryVisibility::for_wayland(false),
            LibraryVisibility::MinimizeAndRestore
        );
    }

    #[test]
    fn it_serializes_wayland_replacement_until_the_new_window_is_open() {
        let (mut windows, _) = Windows::open();
        let library = windows.library_id();
        windows.library_visibility_known(library, true);
        let _ = windows.show_mini();
        let _ = windows.open_wayland_mini(library, super::LIBRARY_SIZE, false);
        let opening = windows.mini_id();

        assert_eq!(windows.toggle_queue().units(), 0);
        assert_eq!(windows.show_library().units(), 0);
        assert_eq!(windows.mini_id(), opening);
    }

    #[test]
    fn it_ignores_a_close_request_queued_for_the_retired_wayland_mini() {
        let (mut windows, _) = Windows::open();
        let library = windows.library_id();
        let _ = windows.show_mini();
        let retired = windows.mini_id().unwrap();
        let _ = windows.library_visibility_checked(retired, true);
        let _ = windows.open_wayland_mini(library, super::LIBRARY_SIZE, false);

        assert_eq!(windows.close_requested(retired).units(), 0);
    }
}
