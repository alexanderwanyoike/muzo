use iced::{
    Event, Font, Pixels, Point, Rectangle, Renderer, Size,
    advanced::{Layout, Shell, clipboard, layout, mouse, widget::Tree},
};

use super::{Desktop, Message, cover_layout_tests::paint, queue_tests::setup};

#[test]
fn it_places_the_transport_display_at_the_original_window_breakpoint() {
    const HEADER_CENTER_Y: usize = 40;
    const WINDOW_HEIGHT: u32 = 800;
    let (mut desktop, _) = setup();
    desktop.appearance.theme = "pulse".into();
    desktop.appearance.mode = "dark".into();
    let border = desktop.appearance.surfaces().line.into_rgba8();

    for (width, display_x, display_width) in [
        (800, 181, 476),
        (850, 181, 526),
        (950, 189, 610),
        (951, 262, 475),
        (1200, 319, 610),
    ] {
        desktop.sidebar.layout_width(width as f32);
        let pixels = paint(
            &desktop,
            desktop.windows.library_id(),
            Size::new(width, WINDOW_HEIGHT),
        );
        let at = |x| {
            let offset = (HEADER_CENTER_Y * width as usize + x) * 4;
            &pixels[offset..offset + 4]
        };
        for x in [display_x, display_x + display_width - 1] {
            assert_eq!(
                at(x),
                border,
                "display edge at ({x}, {HEADER_CENTER_Y}) for {width}px"
            );
        }
        for x in [display_x - 1, display_x + display_width] {
            assert_ne!(at(x), border, "frame outside display at {x} for {width}px");
        }
    }
}

fn click_header(desktop: &mut Desktop, width: f32, position: Point) -> Vec<Message> {
    desktop.sidebar.layout_width(width);
    let mut view = desktop.view(desktop.windows.library_id());
    let mut tree = Tree::new(&view);
    let renderer = Renderer::new(Font::DEFAULT, Pixels(16.0));
    let bounds = Rectangle::with_size(Size::new(width, 800.0));
    let widget = view.as_widget_mut();
    let node = widget.layout(
        &mut tree,
        &renderer,
        &layout::Limits::new(bounds.size(), bounds.size()),
    );
    let mut messages = Vec::new();
    for event in [
        mouse::Event::ButtonPressed(mouse::Button::Left),
        mouse::Event::ButtonReleased(mouse::Button::Left),
    ] {
        widget.update(
            &mut tree,
            &Event::Mouse(event),
            Layout::new(&node),
            mouse::Cursor::Available(position),
            &renderer,
            &mut clipboard::Null,
            &mut Shell::new(&mut messages),
            &bounds,
        );
    }
    messages
}

#[test]
fn it_uses_the_original_volume_slider_width_and_position_in_each_layout() {
    let (mut desktop, _) = setup();
    for (width, x) in [(850.0, 781.0), (1200.0, 1111.33)] {
        let messages = click_header(&mut desktop, width, Point::new(x, 40.0));
        assert!(
            matches!(messages.as_slice(), [Message::SetVolume(value)] if (value - 0.33).abs() < 0.005),
            "click on the reference slider at {width}px gave {messages:?}"
        );
    }
}

#[test]
fn it_keeps_the_original_transport_button_hit_areas_and_actions() {
    let (mut desktop, _) = setup();
    let _ = desktop.update(Message::Select(0));

    let click = |desktop: &mut Desktop, x, y| click_header(desktop, 850.0, Point::new(x, y));
    assert!(matches!(
        click(&mut desktop, 73.5, 24.0).as_slice(),
        [Message::TogglePlayback]
    ));
    assert!(matches!(
        click(&mut desktop, 157.5, 40.0).as_slice(),
        [Message::Next]
    ));
    assert!(
        click(&mut desktop, 26.5, 40.0).is_empty(),
        "Previous is disabled at the first track"
    );
    let _ = desktop.update(Message::Select(1));
    assert!(matches!(
        click(&mut desktop, 26.5, 40.0).as_slice(),
        [Message::Previous]
    ));

    assert!(matches!(
        click(&mut desktop, 811.0, 16.0).as_slice(),
        [Message::ShowMini]
    ));
    assert!(
        click(&mut desktop, 807.0, 16.0).is_empty(),
        "outside the mini player button"
    );
}
