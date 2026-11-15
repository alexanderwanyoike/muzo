use iced::{
    Event, Font, Pixels, Point, Rectangle, Renderer, Size,
    advanced::{Layout, Shell, clipboard, layout, mouse, widget::Tree},
    futures::{FutureExt, StreamExt},
    window,
};
use iced_runtime::{Action, task};

use super::{
    Desktop, Message, cover_layout_tests::paint, queue_tests::setup,
    transport_time_layout_tests::rendered_labels,
};

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

#[cfg(not(target_os = "macos"))]
#[test]
fn it_starts_native_resizing_from_the_edges_of_both_muzo_windows() {
    let (mut desktop, _) = Desktop::with_session(Err(String::new()));
    let _ = desktop.update(Message::ShowMini);
    for (id, size) in [
        (desktop.windows.library_id(), Size::new(1200.0, 800.0)),
        (desktop.windows.mini_id().unwrap(), Size::new(420.0, 560.0)),
    ] {
        let [message]: [Message; 1] =
            click(&desktop, id, size, Point::new(size.width - 2.0, 240.0))
                .try_into()
                .expect("pressing the outside edge must request a native resize");
        let mut actions = task::into_stream(desktop.update(message)).expect("native resize action");
        assert!(matches!(
            actions.next().now_or_never().flatten(),
            Some(Action::Window(window::Action::DragResize(window, window::Direction::East)))
                if window == id
        ));
    }
}

#[test]
fn it_draws_the_requested_mini_layout_in_the_first_frame_at_the_target_size() {
    let compact = Size::new(420.0, 560.0);
    let expanded = Size::new(740.0, 560.0);
    let (mut desktop, _) = Desktop::with_session(Err(String::new()));
    let _ = desktop.update(Message::ShowMini);
    let mini = desktop.windows.mini_id().unwrap();

    let _ = desktop.update(Message::ToggleQueue);
    assert!(!rendered_labels(&desktop, mini, compact).contains_key("Playback queue"));
    assert!(rendered_labels(&desktop, mini, expanded).contains_key("Playback queue"));

    let _ = desktop.update(Message::MiniResized(mini, expanded));
    let _ = desktop.update(Message::ToggleQueue);
    assert!(rendered_labels(&desktop, mini, expanded).contains_key("Playback queue"));
    assert!(!rendered_labels(&desktop, mini, compact).contains_key("Playback queue"));
}

#[test]
fn it_keeps_the_title_bar_inside_the_same_compact_strip_in_both_windows() {
    let (mut desktop, _) = Desktop::with_session(Err(String::new()));
    let _ = desktop.update(Message::ShowMini);
    for (window, size) in [
        (desktop.windows.library_id(), Size::new(1200.0, 800.0)),
        (desktop.windows.mini_id().unwrap(), Size::new(420.0, 560.0)),
    ] {
        assert!(matches!(
            click(&desktop, window, size, Point::new(125.0, 20.0)).as_slice(),
            [Message::DragWindow(target)] if *target == window
        ));
        assert!(
            click(&desktop, window, size, Point::new(125.0, 35.0))
                .iter()
                .all(|message| !matches!(message, Message::DragWindow(_)))
        );
    }
}

#[test]
fn it_draws_separate_solid_rounded_library_panels_in_the_current_theme() {
    let frame = Size::new(850, 800);
    let (mut desktop, _) = setup();
    let window = desktop.windows.library_id();
    desktop.appearance.mode = "dark".into();
    let content_left = desktop.sidebar.visible_width().round() as usize + 1;

    for material in ["pulse", "walnut"] {
        desktop.appearance.theme = material.into();
        let surfaces = desktop.appearance.surfaces();
        let shell = desktop.theme(window).palette().background.into_rgba8();
        let pixels = paint(&desktop, window, frame);
        let at = |x, y| {
            let offset = (y * frame.width as usize + x) * 4;
            &pixels[offset..offset + 4]
        };

        assert_eq!(at(4, 120), shell, "{material} sidebar gutter");
        assert_eq!(at(10, 92), shell, "{material} sidebar corner");
        assert_eq!(
            at(28, 94),
            surfaces.chrome.into_rgba8(),
            "{material} sidebar fill"
        );
        assert_eq!(at(content_left + 2, 92), shell, "{material} content corner");
        assert_eq!(
            at(content_left + 22, 94),
            surfaces.raised.into_rgba8(),
            "{material} content fill"
        );
        assert_eq!(at(846, 120), shell, "{material} content gutter");
    }
}

#[test]
fn it_draws_the_mini_player_and_expanded_queue_in_solid_rounded_theme_panels() {
    for material in ["pulse", "walnut"] {
        let (mut desktop, _) = setup();
        desktop.appearance.theme = material.into();
        desktop.appearance.mode = "dark".into();
        let _ = desktop.update(Message::ShowMini);
        let window = desktop.windows.mini_id().unwrap();
        let surfaces = desktop.appearance.surfaces();
        let shell = desktop.theme(window).palette().background.into_rgba8();

        for (width, left_panel) in [(420, surfaces.raised), (740, surfaces.chrome)] {
            if width == 740 {
                let _ = desktop.update(Message::ToggleQueue);
                let _ = desktop.update(Message::MiniResized(window, Size::new(740.0, 560.0)));
            }
            let pixels = paint(&desktop, window, Size::new(width, 560));
            let at = |x, y| {
                let offset = (y * width as usize + x) * 4;
                &pixels[offset..offset + 4]
            };
            assert_eq!(at(4, 100), shell, "{material} mini gutter");
            assert_eq!(at(10, 42), shell, "{material} mini corner");
            assert_eq!(
                at(28, 44),
                left_panel.into_rgba8(),
                "{material} mini panel fill"
            );
            if width == 740 {
                assert_eq!(at(270, 100), shell, "{material} queue gutter");
                assert_eq!(
                    at(300, 44),
                    surfaces.raised.into_rgba8(),
                    "{material} playback panel fill"
                );
            }
        }
    }
}

#[test]
fn it_paints_the_transport_strip_with_the_same_solid_chrome_as_the_title_bar() {
    let (mut desktop, _) = setup();
    desktop.appearance.theme = "walnut".into();
    desktop.appearance.mode = "dark".into();
    let frame = Size::new(850, 800);
    let pixels = paint(&desktop, desktop.windows.library_id(), frame);
    let chrome = desktop.appearance.surfaces().chrome.into_rgba8();

    for y in [4, 36, 76] {
        let offset = (y * frame.width as usize + 6) * 4;
        assert_eq!(
            &pixels[offset..offset + 4],
            chrome,
            "transport strip at y={y}"
        );
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
