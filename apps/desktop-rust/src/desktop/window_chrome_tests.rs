use iced::{
    Event, Font, Pixels, Point, Rectangle, Renderer, Size,
    advanced::{Layout, Shell, clipboard, layout, mouse, widget::Tree},
    futures::{FutureExt, StreamExt},
    window,
};
use iced_runtime::{Action, task};

use super::{Desktop, Message};

#[test]
fn it_draws_the_close_control_inside_both_muzo_windows() {
    let (mut desktop, open) = Desktop::with_session(Err(String::new()));
    let library = desktop.windows.library_id();
    let mini_open = desktop.update(Message::ShowMini);
    let mini = desktop.windows.mini_id().unwrap();

    for (id, open, size) in [
        (library, open, Size::new(1200.0, 800.0)),
        (mini, mini_open, Size::new(420.0, 560.0)),
    ] {
        let mut actions = task::into_stream(open).expect("window opens");
        let Some(Action::Window(window::Action::Open(opened, settings, _))) =
            actions.next().now_or_never().flatten()
        else {
            panic!("expected the window open action");
        };
        assert_eq!(opened, id);
        let messages = click(&desktop, id, size, Point::new(size.width - 17.0, 20.0));
        assert!(
            matches!(messages.as_slice(), [Message::CloseRequested(window)] if *window == id),
            "the upper-right close control should act on {id:?}, got {messages:?}"
        );
        assert!(!settings.decorations);
    }
}

fn click(desktop: &Desktop, id: window::Id, size: Size, point: Point) -> Vec<Message> {
    let mut view = desktop.view(id);
    let mut tree = Tree::new(&view);
    let renderer = Renderer::new(Font::DEFAULT, Pixels(16.0));
    let widget = view.as_widget_mut();
    let node = widget.layout(&mut tree, &renderer, &layout::Limits::new(size, size));
    let mut messages = Vec::new();
    for event in [
        mouse::Event::ButtonPressed(mouse::Button::Left),
        mouse::Event::ButtonReleased(mouse::Button::Left),
    ] {
        widget.update(
            &mut tree,
            &Event::Mouse(event),
            Layout::new(&node),
            mouse::Cursor::Available(point),
            &renderer,
            &mut clipboard::Null,
            &mut Shell::new(&mut messages),
            &Rectangle::with_size(size),
        );
    }
    messages
}
