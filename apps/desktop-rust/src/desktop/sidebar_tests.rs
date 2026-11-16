use iced::{
    Event, Font, Pixels, Point, Rectangle, Renderer, Size, Theme,
    advanced::{
        Layout, Shell, Widget, clipboard, layout, mouse,
        widget::{Tree, operation},
    },
    futures::{FutureExt, StreamExt},
    keyboard::{
        Key, Location, Modifiers,
        key::{Named, NativeCode, Physical},
    },
};
use iced_runtime::{Action, task};

use crate::appearance::sidebar::{SidebarWidth, load_or_import, save};

use super::{CataloguePaths, Desktop, LibraryPage, Message, Sources, input_message};

const VIEW_WIDTH: f32 = 800.0;
const VIEW_HEIGHT: f32 = 600.0;
const DRAG_HEIGHT: f32 = 260.0;
const NARROW_DIVIDER: Point = Point::new(279.5, DRAG_HEIGHT);

struct Library {
    desktop: Desktop,
    renderer: Renderer,
    tree: Tree,
    width: f32,
}

impl Library {
    fn new(paths: CataloguePaths) -> Self {
        let (mut desktop, _) = Desktop::with_session(Err("Test output".into()));
        desktop.sidebar.set(SidebarWidth::new(340.0));
        desktop.sidebar.layout_width(VIEW_WIDTH);
        desktop.sources = Sources::new(Some(paths));
        Self {
            desktop,
            renderer: Renderer::new(Font::DEFAULT, Pixels(16.0)),
            tree: Tree::empty(),
            width: VIEW_WIDTH,
        }
    }

    fn input(&mut self, event: Event, x: f32) {
        self.batch(&[event], Point::new(x, DRAG_HEIGHT));
    }

    fn batch(&mut self, events: &[Event], cursor: Point) {
        for message in self.messages(events, cursor) {
            let _ = self.desktop.update(message);
        }
    }

    fn messages(&mut self, events: &[Event], cursor: Point) -> Vec<Message> {
        let id = self.desktop.windows.library_id();
        let mut messages = Vec::new();
        let mut view = self.desktop.view(id);
        let widget = view.as_widget_mut();
        let bounds = Rectangle::with_size(Size::new(self.width, VIEW_HEIGHT));
        let node = layout_library(widget, &mut self.tree, &self.renderer, bounds.size());
        for event in events {
            let status = {
                let mut shell = Shell::new(&mut messages);
                widget.update(
                    &mut self.tree,
                    event,
                    Layout::new(&node),
                    mouse::Cursor::Available(cursor),
                    &self.renderer,
                    &mut clipboard::Null,
                    &mut shell,
                    &bounds,
                );
                shell.event_status()
            };
            messages.extend(input_message(event.clone(), status, id));
        }
        messages
    }

    fn drag_inward(&mut self) {
        self.input(
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)),
            279.5,
        );
        self.input(
            Event::Mouse(mouse::Event::CursorMoved {
                position: Point::new(245.5, DRAG_HEIGHT),
            }),
            245.5,
        );
        assert_eq!(self.desktop.sidebar.visible_width(), 245.0);
    }

    fn release(&mut self) {
        self.input(
            Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)),
            245.5,
        );
    }

    fn resize(&mut self, width: f32) {
        self.width = width;
        self.input(
            Event::Window(iced::window::Event::Resized(Size::new(width, VIEW_HEIGHT))),
            0.0,
        );
    }

    fn tab(&mut self, backwards: bool) {
        let id = self.desktop.windows.library_id();
        let tab = key(
            Named::Tab,
            if backwards {
                Modifiers::SHIFT
            } else {
                Modifiers::empty()
            },
        );
        let [message]: [Message; 1] = self
            .messages(&[tab], Point::ORIGIN)
            .try_into()
            .expect("one message for a Tab key press");
        let tasks = self.desktop.update(message);
        let mut actions = task::into_stream(tasks).expect("Tab schedules a focus change");
        let mut view = self.desktop.view(id);
        let widget = view.as_widget_mut();
        let node = layout_library(
            widget,
            &mut self.tree,
            &self.renderer,
            Size::new(self.width, VIEW_HEIGHT),
        );
        while let Some(action) = actions
            .next()
            .now_or_never()
            .expect("a ready widget action")
        {
            let Action::Widget(mut operation) = action else {
                panic!("Tab should only operate on the library widgets");
            };
            loop {
                widget.operate(
                    &mut self.tree,
                    Layout::new(&node),
                    &self.renderer,
                    operation.as_mut(),
                );
                let operation::Outcome::Chain(next) = operation.finish() else {
                    break;
                };
                operation = next;
            }
        }
    }
}

fn layout_library(
    widget: &mut dyn Widget<Message, Theme, Renderer>,
    tree: &mut Tree,
    renderer: &Renderer,
    size: Size,
) -> layout::Node {
    tree.diff(&*widget);
    widget.layout(tree, renderer, &layout::Limits::new(size, size))
}

#[test]
fn it_saves_the_sidebar_drag_only_on_release() {
    let temp = tempfile::tempdir().unwrap();
    let paths = CataloguePaths::in_directory(temp.path());
    let saved = SidebarWidth::new(340.0);
    save(&paths.sidebar(), saved).unwrap();
    let mut library = Library::new(paths.clone());

    library.drag_inward();

    assert_eq!(library.desktop.sidebar.preferred_width(), saved);
    assert_eq!(load_or_import(&paths.sidebar(), None).unwrap(), saved);
    library.release();
    let committed = SidebarWidth::new(245.0);
    assert_eq!(library.desktop.sidebar.preferred_width(), committed);
    assert_eq!(load_or_import(&paths.sidebar(), None).unwrap(), committed);
}

fn click_at(position: Point) -> [Event; 3] {
    [
        Event::Mouse(mouse::Event::CursorMoved { position }),
        Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)),
        Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)),
    ]
}

fn key(name: Named, modifiers: Modifiers) -> Event {
    Event::Keyboard(iced::keyboard::Event::KeyPressed {
        key: Key::Named(name),
        modified_key: Key::Named(name),
        physical_key: Physical::Unidentified(NativeCode::Unidentified),
        location: Location::Standard,
        modifiers,
        text: None,
        repeat: false,
    })
}

#[test]
fn it_accumulates_sidebar_arrow_steps_and_saves_the_viewport_limits() {
    let temp = tempfile::tempdir().unwrap();
    let paths = CataloguePaths::in_directory(temp.path());
    let initial = SidebarWidth::new(340.0);
    save(&paths.sidebar(), initial).unwrap();
    let mut library = Library::new(paths.clone());
    let arrow = key(Named::ArrowLeft, Modifiers::empty());

    library.batch(std::slice::from_ref(&arrow), NARROW_DIVIDER);
    assert_eq!(load_or_import(&paths.sidebar(), None).unwrap(), initial);

    library.batch(&click_at(NARROW_DIVIDER), NARROW_DIVIDER);
    library.batch(&[arrow.clone(), arrow], NARROW_DIVIDER);
    assert_eq!(library.desktop.sidebar.visible_width(), 263.0);
    assert_eq!(
        load_or_import(&paths.sidebar(), None).unwrap(),
        SidebarWidth::new(263.0)
    );

    library.input(key(Named::Home, Modifiers::empty()), 0.0);
    assert_eq!(library.desktop.sidebar.visible_width(), 192.0);
    library.input(key(Named::End, Modifiers::empty()), 0.0);
    assert_eq!(library.desktop.sidebar.visible_width(), 279.0);
    library.resize(1200.0);
    library.input(key(Named::End, Modifiers::empty()), 0.0);
    assert_eq!(
        load_or_import(&paths.sidebar(), None).unwrap(),
        SidebarWidth::new(360.0)
    );
}

#[test]
fn it_takes_and_leaves_divider_focus_in_both_library_panels() {
    for page in [LibraryPage::Songs, LibraryPage::Playlists] {
        let temp = tempfile::tempdir().unwrap();
        let mut library = Library::new(CataloguePaths::in_directory(temp.path()));
        library.desktop.page = page;
        library.desktop.sources.snapshot = Some(super::queue_tests::snapshot(&["source"]));
        let left = key(Named::ArrowLeft, Modifiers::empty());

        library.tab(false);
        library.input(left.clone(), 0.0);
        assert_eq!(library.desktop.sidebar.visible_width(), 279.0);

        if page == LibraryPage::Songs {
            for _ in 0..2 {
                library.tab(false);
                library.input(left.clone(), 0.0);
                assert_eq!(library.desktop.sidebar.visible_width(), 279.0);
            }
        }

        library.tab(false);
        library.input(left.clone(), 0.0);
        assert_eq!(library.desktop.sidebar.visible_width(), 271.0);

        library.tab(false);
        library.input(left.clone(), 0.0);
        assert_eq!(library.desktop.sidebar.visible_width(), 271.0);

        library.tab(true);
        library.input(left, 0.0);
        assert_eq!(library.desktop.sidebar.visible_width(), 263.0);
    }
}

#[test]
fn it_reaches_the_sidebar_divider_in_settings_and_returns_to_appearance() {
    let temp = tempfile::tempdir().unwrap();
    let mut library = Library::new(CataloguePaths::in_directory(temp.path()));
    library.desktop.page = LibraryPage::Settings;
    let initial_accent = library.desktop.appearance.accent.clone();
    let enter = key(Named::Enter, Modifiers::empty());

    library.tab(true);
    library.input(key(Named::Home, Modifiers::empty()), 0.0);
    assert_eq!(library.desktop.sidebar.visible_width(), 192.0);
    library.input(enter.clone(), 0.0);
    assert_eq!(library.desktop.appearance.accent, initial_accent);

    let divider = Point::new(192.5, DRAG_HEIGHT);
    library.batch(&click_at(divider), divider);
    library.tab(true);
    library.input(key(Named::End, Modifiers::empty()), 0.0);
    assert_eq!(library.desktop.sidebar.visible_width(), 192.0);
    library.input(enter, 0.0);
    assert_eq!(library.desktop.appearance.accent, "glow-orange");
}

#[test]
fn it_commits_the_last_pointer_width_after_a_keyboard_resize_during_a_drag() {
    let temp = tempfile::tempdir().unwrap();
    let paths = CataloguePaths::in_directory(temp.path());
    let mut library = Library::new(paths.clone());
    library.drag_inward();

    library.input(key(Named::Home, Modifiers::empty()), 245.5);
    assert_eq!(library.desktop.sidebar.visible_width(), 192.0);
    library.release();

    assert_eq!(library.desktop.sidebar.visible_width(), 245.0);
    assert_eq!(
        load_or_import(&paths.sidebar(), None).unwrap(),
        SidebarWidth::new(245.0)
    );
}

#[test]
fn it_saves_the_default_sidebar_width_after_two_completed_clicks() {
    let temp = tempfile::tempdir().unwrap();
    let paths = CataloguePaths::in_directory(temp.path());
    save(&paths.sidebar(), SidebarWidth::new(340.0)).unwrap();
    let mut library = Library::new(paths.clone());
    let [movement, press, release] = click_at(NARROW_DIVIDER);

    library.batch(
        &[movement, press.clone(), release.clone(), press, release],
        NARROW_DIVIDER,
    );
    library.resize(1200.0);

    assert_eq!(library.desktop.sidebar.visible_width(), 240.0);
    assert_eq!(
        load_or_import(&paths.sidebar(), None).unwrap(),
        SidebarWidth::default()
    );
}

#[test]
fn it_resets_the_sidebar_when_the_second_click_moves_vertically() {
    let temp = tempfile::tempdir().unwrap();
    let paths = CataloguePaths::in_directory(temp.path());
    let previous = SidebarWidth::new(340.0);
    save(&paths.sidebar(), previous).unwrap();
    let mut library = Library::new(paths.clone());
    let [movement, press, release] = click_at(NARROW_DIVIDER);
    let lower = Point::new(NARROW_DIVIDER.x, NARROW_DIVIDER.y + 1.0);

    library.batch(
        &[
            movement,
            press.clone(),
            release.clone(),
            press,
            Event::Mouse(mouse::Event::CursorMoved { position: lower }),
            release,
        ],
        lower,
    );

    assert_eq!(
        load_or_import(&paths.sidebar(), None).unwrap(),
        SidebarWidth::default()
    );
}

#[test]
fn it_treats_clicks_interrupted_by_another_target_as_separate() {
    let temp = tempfile::tempdir().unwrap();
    let paths = CataloguePaths::in_directory(temp.path());
    let previous = SidebarWidth::new(340.0);
    save(&paths.sidebar(), previous).unwrap();
    let mut library = Library::new(paths.clone());
    let outside = Point::new(500.0, 480.0);
    let events = [
        click_at(NARROW_DIVIDER),
        click_at(outside),
        click_at(NARROW_DIVIDER),
    ]
    .concat();

    library.batch(&events, NARROW_DIVIDER);

    assert_eq!(library.desktop.sidebar.preferred_width(), previous);
    assert_eq!(load_or_import(&paths.sidebar(), None).unwrap(), previous);
}

#[test]
fn it_commits_a_returning_drag_after_a_click_without_resetting_the_sidebar() {
    let temp = tempfile::tempdir().unwrap();
    let paths = CataloguePaths::in_directory(temp.path());
    let mut library = Library::new(paths.clone());
    let [movement, press, release] = click_at(NARROW_DIVIDER);

    library.batch(
        &[
            movement.clone(),
            press.clone(),
            release.clone(),
            press,
            Event::Mouse(mouse::Event::CursorMoved {
                position: Point::new(245.5, DRAG_HEIGHT),
            }),
            movement,
            release,
        ],
        NARROW_DIVIDER,
    );

    assert_eq!(
        load_or_import(&paths.sidebar(), None).unwrap(),
        SidebarWidth::new(279.0)
    );
}

#[test]
fn it_restores_the_preferred_sidebar_width_after_cancelling_a_narrow_drag() {
    let temp = tempfile::tempdir().unwrap();
    let paths = CataloguePaths::in_directory(temp.path());
    let saved = SidebarWidth::new(340.0);
    save(&paths.sidebar(), saved).unwrap();
    let mut library = Library::new(paths.clone());
    library.drag_inward();

    library.input(
        Event::Keyboard(iced::keyboard::Event::KeyPressed {
            key: Key::Named(Named::Escape),
            modified_key: Key::Named(Named::Escape),
            physical_key: Physical::Unidentified(NativeCode::Unidentified),
            location: Location::Standard,
            modifiers: Modifiers::empty(),
            text: None,
            repeat: false,
        }),
        245.5,
    );
    library.release();
    assert_eq!(library.desktop.sidebar.visible_width(), 279.0);
    library.resize(1200.0);

    assert_eq!(library.desktop.sidebar.visible_width(), 340.0);
    assert_eq!(load_or_import(&paths.sidebar(), None).unwrap(), saved);
}

#[test]
fn it_preserves_the_drag_target_until_release_when_widening_a_narrow_window() {
    let temp = tempfile::tempdir().unwrap();
    let paths = CataloguePaths::in_directory(temp.path());
    let mut library = Library::new(paths.clone());
    library.input(
        Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)),
        279.5,
    );
    library.input(
        Event::Mouse(mouse::Event::CursorMoved {
            position: Point::new(310.5, DRAG_HEIGHT),
        }),
        310.5,
    );

    assert_eq!(library.desktop.sidebar.visible_width(), 279.0);
    library.resize(1200.0);
    assert_eq!(library.desktop.sidebar.visible_width(), 310.0);
    library.release();
    assert_eq!(
        load_or_import(&paths.sidebar(), None).unwrap(),
        SidebarWidth::new(310.0)
    );
}

#[test]
fn it_commits_a_real_drag_back_to_the_narrow_window_limit() {
    let temp = tempfile::tempdir().unwrap();
    let paths = CataloguePaths::in_directory(temp.path());
    let mut library = Library::new(paths.clone());
    library.drag_inward();
    library.input(
        Event::Mouse(mouse::Event::CursorMoved {
            position: Point::new(279.5, DRAG_HEIGHT),
        }),
        279.5,
    );
    library.release();
    library.resize(1200.0);

    assert_eq!(library.desktop.sidebar.visible_width(), 279.0);
    assert_eq!(
        load_or_import(&paths.sidebar(), None).unwrap(),
        SidebarWidth::new(279.0)
    );
}

#[test]
fn it_starts_at_the_divider_when_mouse_press_and_movement_share_a_cursor_sample() {
    let temp = tempfile::tempdir().unwrap();
    let mut library = Library::new(CataloguePaths::in_directory(temp.path()));
    library.input(
        Event::Mouse(mouse::Event::CursorMoved {
            position: Point::new(279.5, DRAG_HEIGHT),
        }),
        245.5,
    );
    library.input(
        Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)),
        245.5,
    );
    library.input(
        Event::Mouse(mouse::Event::CursorMoved {
            position: Point::new(245.5, DRAG_HEIGHT),
        }),
        245.5,
    );

    assert_eq!(library.desktop.sidebar.visible_width(), 245.0);
    library.input(Event::Window(iced::window::Event::Unfocused), 245.5);
    library.release();
    assert_eq!(library.desktop.sidebar.visible_width(), 279.0);
    assert_eq!(
        library.desktop.sidebar.preferred_width(),
        SidebarWidth::new(340.0)
    );
}

#[test]
fn it_tracks_the_initiating_finger_and_cancels_when_touch_is_lost() {
    use iced::touch::{Event as Touch, Finger};

    let temp = tempfile::tempdir().unwrap();
    let mut library = Library::new(CataloguePaths::in_directory(temp.path()));
    library.input(
        Event::Touch(Touch::FingerPressed {
            id: Finger(1),
            position: Point::new(279.5, DRAG_HEIGHT),
        }),
        440.0,
    );
    library.input(
        Event::Touch(Touch::FingerMoved {
            id: Finger(1),
            position: Point::new(245.5, DRAG_HEIGHT),
        }),
        440.0,
    );
    library.input(
        Event::Touch(Touch::FingerMoved {
            id: Finger(2),
            position: Point::new(440.0, DRAG_HEIGHT),
        }),
        440.0,
    );

    assert_eq!(library.desktop.sidebar.visible_width(), 245.0);
    library.input(
        Event::Touch(Touch::FingerLost {
            id: Finger(1),
            position: Point::new(245.5, DRAG_HEIGHT),
        }),
        440.0,
    );
    assert_eq!(library.desktop.sidebar.visible_width(), 279.0);
    assert_eq!(
        library.desktop.sidebar.preferred_width(),
        SidebarWidth::new(340.0)
    );
}

#[test]
fn it_uses_the_latest_window_width_when_resize_and_release_share_a_batch() {
    let temp = tempfile::tempdir().unwrap();
    let paths = CataloguePaths::in_directory(temp.path());
    let mut library = Library::new(paths.clone());
    let target = Point::new(310.5, DRAG_HEIGHT);
    library.input(
        Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)),
        279.5,
    );
    library.input(
        Event::Mouse(mouse::Event::CursorMoved { position: target }),
        target.x,
    );

    library.batch(
        &[
            Event::Window(iced::window::Event::Resized(Size::new(1200.0, 600.0))),
            Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)),
        ],
        target,
    );

    assert_eq!(
        load_or_import(&paths.sidebar(), None).unwrap(),
        SidebarWidth::new(310.0)
    );
}

#[test]
fn it_keeps_a_batched_divider_drag_from_activating_the_navigation_under_it() {
    let temp = tempfile::tempdir().unwrap();
    let mut library = Library::new(CataloguePaths::in_directory(temp.path()));
    let divider = Point::new(279.5, 230.0);
    let over_navigation = Point::new(245.5, divider.y);

    library.batch(
        &[
            Event::Mouse(mouse::Event::CursorMoved { position: divider }),
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)),
            Event::Mouse(mouse::Event::CursorMoved {
                position: over_navigation,
            }),
            Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)),
        ],
        over_navigation,
    );

    assert_eq!(library.desktop.sidebar.visible_width(), 245.0);
    assert_eq!(library.desktop.page, LibraryPage::Songs);
}

#[test]
fn it_keeps_a_batched_divider_drag_from_opening_the_mini_player() {
    let temp = tempfile::tempdir().unwrap();
    let mut library = Library::new(CataloguePaths::in_directory(temp.path()));
    let divider = Point::new(279.5, DRAG_HEIGHT);
    let mini_button = Point::new(774.0, 16.0);

    library.batch(
        &[
            Event::Mouse(mouse::Event::CursorMoved { position: divider }),
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)),
            Event::Mouse(mouse::Event::CursorMoved {
                position: mini_button,
            }),
            Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)),
        ],
        mini_button,
    );

    assert_eq!(
        library.desktop.sidebar.preferred_width(),
        SidebarWidth::new(279.0)
    );
    assert!(library.desktop.windows.mini_id().is_none());
}

#[test]
fn it_draws_the_hovered_sidebar_divider_with_the_glow_ink() {
    use iced::{
        Color,
        advanced::renderer::{Headless, Style},
    };

    let temp = tempfile::tempdir().unwrap();
    let mut library = Library::new(CataloguePaths::in_directory(temp.path()));
    library.desktop.appearance.accent = "glow-cyan".into();
    let id = library.desktop.windows.library_id();
    let pixels = Size::new(800, 600);
    let viewport = Rectangle::with_size(Size::new(pixels.width as f32, pixels.height as f32));
    let sample = (420 * pixels.width + 278) as usize * 4;

    for (mode, ink) in [("light", [0, 95, 109, 255]), ("dark", [66, 233, 255, 255])] {
        library.desktop.appearance.mode = mode.into();
        let theme = library.desktop.theme(id);
        let mut view = library.desktop.view(id);
        let mut renderer = Renderer::new(Font::DEFAULT, Pixels(16.0));
        let mut tree = Tree::new(&view);
        let widget = view.as_widget_mut();
        let node = widget.layout(
            &mut tree,
            &renderer,
            &layout::Limits::new(viewport.size(), viewport.size()),
        );
        widget.draw(
            &tree,
            &mut renderer,
            &theme,
            &Style::default(),
            Layout::new(&node),
            mouse::Cursor::Available(Point::new(279.5, 420.0)),
            &viewport,
        );
        let frame = renderer.screenshot(pixels, 1.0, Color::WHITE);

        assert_eq!(&frame[sample..sample + 4], &ink, "unexpected ink in {mode}");
    }
}
