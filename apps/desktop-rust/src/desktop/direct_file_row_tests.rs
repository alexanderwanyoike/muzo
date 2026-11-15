use iced::{
    Event, Font, Pixels, Rectangle, Renderer, Size,
    advanced::{
        Layout, Shell, clipboard, layout, mouse,
        widget::{Id, Operation, Tree},
    },
};

use crate::track::Track;

use super::{
    Desktop, Message,
    queue_tests::{setup, snapshot},
    transport_time_layout_tests::rendered_labels,
};

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

#[test]
fn it_keeps_a_direct_file_playable_after_clearing_a_source_from_a_mixed_queue() {
    const SOURCE: &str = "indexed-library";
    const INDEXED_TITLE: &str = "Indexed song";
    const DIRECT_TITLE: &str = "Opened file";
    let (mut desktop, fixture) = setup();
    let local_path = fixture.path().to_owned();
    let indexed = Track::new(local_path.clone(), INDEXED_TITLE.into(), None, None, None)
        .with_library(SOURCE.into())
        .with_indexed_id("indexed-track".into());
    let direct = Track::new(local_path, DIRECT_TITLE.into(), None, None, None);
    desktop
        .session
        .as_mut()
        .unwrap()
        .replace(vec![indexed, direct]);
    let mut source = snapshot(&[SOURCE]);
    source.selected = Some(SOURCE.into());
    desktop.sources.snapshot = Some(source);

    let _ = desktop.update(Message::ClearSourceSelection(SOURCE.into()));
    let labels = rendered_labels(&desktop, desktop.windows.library_id(), LIBRARY_FRAME);
    assert!(
        !labels.contains_key(INDEXED_TITLE),
        "the indexed song must stay out of Songs"
    );
    click_direct_file(&mut desktop, DIRECT_TITLE);

    let session = desktop.session.as_ref().unwrap();
    assert_eq!(session.current_index(), Some(1));
    assert_eq!(session.current().unwrap().title(), DIRECT_TITLE);
    assert!(session.is_playing());
}
