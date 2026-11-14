use iced::{
    Event, Font, Pixels, Point, Rectangle, Renderer, Size,
    advanced::{Layout, Shell, clipboard, layout, mouse, widget::Tree},
    keyboard::{
        Key, Location, Modifiers,
        key::{Named, NativeCode, Physical},
    },
};

use crate::appearance::sidebar::{SidebarWidth, load_or_import, save};

use super::{CataloguePaths, Desktop, Message, Sources, input_message};

const VIEW_WIDTH: f32 = 800.0;
const DRAG_HEIGHT: f32 = 260.0;

struct Library {
    desktop: Desktop,
    renderer: Renderer,
    tree: Tree,
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
        }
    }

    fn input(&mut self, event: Event, x: f32) {
        let id = self.desktop.windows.library_id();
        let mut messages = Vec::new();
        let status = {
            let mut view = self.desktop.view(id);
            let widget = view.as_widget_mut();
            self.tree.diff(&*widget);
            let bounds = Rectangle::with_size(Size::new(VIEW_WIDTH, 600.0));
            let node = widget.layout(
                &mut self.tree,
                &self.renderer,
                &layout::Limits::new(bounds.size(), bounds.size()),
            );
            let mut shell = Shell::new(&mut messages);
            widget.update(
                &mut self.tree,
                &event,
                Layout::new(&node),
                mouse::Cursor::Available(Point::new(x, DRAG_HEIGHT)),
                &self.renderer,
                &mut clipboard::Null,
                &mut shell,
                &bounds,
            );
            shell.event_status()
        };
        messages.extend(input_message(event, status, id));
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
    let id = library.desktop.windows.library_id();
    let _ = library
        .desktop
        .update(Message::MiniResized(id, Size::new(1200.0, 800.0)));

    assert_eq!(library.desktop.sidebar.visible_width(), 340.0);
    assert_eq!(load_or_import(&paths.sidebar(), None).unwrap(), saved);
}
