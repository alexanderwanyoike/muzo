use iced::{
    Event, Font, Pixels, Point, Rectangle, Renderer, Size,
    advanced::{Layout, Shell, clipboard, layout, mouse, widget::Tree},
    keyboard::{
        Key, Location, Modifiers,
        key::{Named, NativeCode, Physical},
    },
};

use crate::appearance::sidebar::{SidebarWidth, load_or_import, save};

use super::{CataloguePaths, Desktop, LibraryPage, Sources, input_message};

const VIEW_WIDTH: f32 = 800.0;
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
        let id = self.desktop.windows.library_id();
        let mut messages = Vec::new();
        {
            let mut view = self.desktop.view(id);
            let widget = view.as_widget_mut();
            self.tree.diff(&*widget);
            let bounds = Rectangle::with_size(Size::new(self.width, 600.0));
            let node = widget.layout(
                &mut self.tree,
                &self.renderer,
                &layout::Limits::new(bounds.size(), bounds.size()),
            );
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
        }
        for message in messages {
            let _ = self.desktop.update(message);
        }
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
            Event::Window(iced::window::Event::Resized(Size::new(width, 600.0))),
            0.0,
        );
    }
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
