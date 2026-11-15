use iced::{
    Event, Font, Pixels, Rectangle, Renderer, Size,
    advanced::{
        Layout, Shell, clipboard, layout, mouse,
        widget::{Id, Operation, Tree},
    },
};

use crate::track::Track;

use super::{Desktop, Message, queue_tests::setup};

const LIBRARY_FRAME: Size = Size::new(1200.0, 800.0);
const CURRENT_FILE_TITLE: &str = "First direct file";

struct DirectFileRow<'a> {
    title: &'a str,
    bounds: Option<Rectangle>,
}

impl Operation for DirectFileRow<'_> {
    fn traverse(&mut self, visit: &mut dyn FnMut(&mut dyn Operation)) {
        visit(self);
    }

    fn text(&mut self, _: Option<&Id>, bounds: Rectangle, label: &str) {
        // The transport shows the same title above the file row.
        if label == self.title && self.bounds.is_none_or(|previous| bounds.y > previous.y) {
            self.bounds = Some(bounds);
        }
    }
}

fn click_direct_file(desktop: &mut Desktop, title: &str) {
    let mut messages = Vec::new();
    let renderer = Renderer::new(Font::DEFAULT, Pixels(16.0));
    let bounds = Rectangle::with_size(LIBRARY_FRAME);
    {
        let mut view = desktop.view(desktop.windows.library_id());
        let mut tree = Tree::new(&view);
        let widget = view.as_widget_mut();
        let node = widget.layout(
            &mut tree,
            &renderer,
            &layout::Limits::new(bounds.size(), bounds.size()),
        );
        let layout = Layout::new(&node);
        let mut row = DirectFileRow {
            title,
            bounds: None,
        };
        widget.operate(&mut tree, layout, &renderer, &mut row);
        let cursor =
            mouse::Cursor::Available(row.bounds.expect("visible direct-file row").center());
        for event in [
            mouse::Event::ButtonPressed(mouse::Button::Left),
            mouse::Event::ButtonReleased(mouse::Button::Left),
        ] {
            widget.update(
                &mut tree,
                &Event::Mouse(event),
                layout,
                cursor,
                &renderer,
                &mut clipboard::Null,
                &mut Shell::new(&mut messages),
                &bounds,
            );
        }
    }
    for message in messages {
        let _ = desktop.update(message);
    }
}

#[test]
fn it_pauses_and_resumes_the_current_direct_file_from_its_library_row() {
    let (mut desktop, fixture) = setup();
    let direct = Track::new(
        fixture.path().to_owned(),
        CURRENT_FILE_TITLE.into(),
        None,
        None,
        None,
    );
    desktop
        .session
        .as_mut()
        .unwrap()
        .replace_track(0, direct)
        .unwrap();
    let _ = desktop.update(Message::Select(0));
    let session = desktop.session.as_ref().unwrap();
    assert!(session.is_playing());
    let generation = session.start_generation();

    click_direct_file(&mut desktop, CURRENT_FILE_TITLE);
    assert!(!desktop.session.as_ref().unwrap().is_playing());

    click_direct_file(&mut desktop, CURRENT_FILE_TITLE);
    let session = desktop.session.as_ref().unwrap();
    assert!(session.is_playing());
    assert_eq!(session.current_index(), Some(0));
    assert_eq!(session.start_generation(), generation);
}
