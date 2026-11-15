use iced::{
    Event, Font, Pixels, Point, Rectangle, Renderer, Size,
    advanced::{Layout, Shell, clipboard, layout, mouse, widget::Tree},
};

use crate::track::Track;

use super::{
    Desktop, Message, cover_layout_tests::paint, queue_tests::setup,
    transport_time_layout_tests::rendered_labels,
};

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
    header_input(
        desktop,
        width,
        [
            (position, mouse::Event::ButtonPressed(mouse::Button::Left)),
            (position, mouse::Event::ButtonReleased(mouse::Button::Left)),
        ],
    )
}

fn header_input(
    desktop: &mut Desktop,
    width: f32,
    events: impl IntoIterator<Item = (Point, mouse::Event)>,
) -> Vec<Message> {
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
    for (position, event) in events {
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
fn it_uses_the_original_volume_slider_geometry_and_five_percent_steps() {
    let (mut desktop, _) = setup();
    for (width, x) in [(850.0, 784.0), (1200.0, 1114.0)] {
        let messages = click_header(&mut desktop, width, Point::new(x, 34.0));
        assert!(
            matches!(messages.as_slice(), [Message::SetVolume(value)] if (value - 0.35).abs() < 0.005),
            "click on the reference slider at {width}px gave {messages:?}"
        );
        assert!(click_header(&mut desktop, width, Point::new(x, 29.0)).is_empty());
    }
}

#[test]
fn it_maps_volume_along_the_thumb_travel_without_jumping_when_grabbed() {
    const REFERENCE_VOLUME: f32 = 0.30;
    let (mut desktop, _) = setup();
    for (width, rail_click, thumb_center) in [(850.0, 781.0, 781.3), (1200.0, 1111.0, 1111.0)] {
        let _ = desktop.update(Message::SetVolume(1.0));
        let messages = click_header(&mut desktop, width, Point::new(rail_click, 34.0));
        assert!(
            matches!(messages.as_slice(), [Message::SetVolume(value)] if (value - REFERENCE_VOLUME).abs() < 0.005),
            "click on the original thumb travel at {width}px gave {messages:?}"
        );

        let _ = desktop.update(Message::SetVolume(REFERENCE_VOLUME));
        for offset in [0.0, 3.7] {
            let grab = click_header(&mut desktop, width, Point::new(thumb_center + offset, 40.0));
            assert!(
                grab.is_empty(),
                "grabbing the thumb at {width}px and offset {offset} changed the volume: {grab:?}"
            );
        }
        let _ = desktop.update(Message::SetVolume(0.32));
        let grab = click_header(&mut desktop, width, Point::new(thumb_center + 1.5, 40.0));
        assert!(
            grab.is_empty(),
            "grabbing the existing mini-player volume changed it: {grab:?}"
        );
    }
}

#[test]
fn it_keeps_the_volume_at_maximum_when_a_drag_crosses_the_mini_launcher() {
    let (mut desktop, _) = setup();
    let _ = desktop.update(Message::SetVolume(0.50));
    let rail = Point::new(826.0, 40.0);
    let launcher = Point::new(826.0, 16.0);
    let messages = header_input(
        &mut desktop,
        850.0,
        [
            (rail, mouse::Event::ButtonPressed(mouse::Button::Left)),
            (launcher, mouse::Event::CursorMoved { position: launcher }),
            (launcher, mouse::Event::ButtonReleased(mouse::Button::Left)),
        ],
    );
    assert!(
        matches!(messages.as_slice(), [Message::SetVolume(value)] if *value == 1.0),
        "vertical drag changed the volume: {messages:?}"
    );
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

#[test]
fn it_abbreviates_now_playing_labels_inside_the_original_display_column() {
    const TITLE: &str = "An unusually long recording from the coast containing the full rehearsals of a musical ensemble and studio outtakes recorded on two subsequent summer afternoons";
    const ARTIST: &str = "The North Atlantic field recording assembly with orchestral musicians scattered through the streets of the city and several rural villages";
    let (mut desktop, local) = setup();
    desktop
        .session
        .as_mut()
        .unwrap()
        .replace_track(
            0,
            Track::new(
                local.path().to_owned(),
                TITLE.into(),
                Some(ARTIST.into()),
                None,
                None,
            ),
        )
        .unwrap();
    let _ = desktop.update(Message::Select(0));
    desktop.sidebar.layout_width(850.0);
    let labels = rendered_labels(
        &desktop,
        desktop.windows.library_id(),
        Size::new(850.0, 800.0),
    );

    for (full, expected_top) in [(TITLE, 19.0), (ARTIST, 34.4)] {
        let (_, bounds) = labels
            .iter()
            .find(|(label, _)| {
                label
                    .strip_suffix('…')
                    .is_some_and(|prefix| !prefix.is_empty() && full.starts_with(prefix))
            })
            .unwrap_or_else(|| panic!("no abbreviated now-playing label for {full:?}"));
        assert!(
            (bounds.x - 252.0).abs() < 0.5,
            "metadata column: {bounds:?}"
        );
        assert!(
            (bounds.width - 442.0).abs() < 0.5,
            "metadata width: {bounds:?}"
        );
        assert!(
            (bounds.y - expected_top).abs() < 0.8,
            "metadata line position: {bounds:?}"
        );
    }
}

#[test]
fn it_draws_the_original_glowing_play_control_when_a_track_is_available() {
    const SIZE: Size<u32> = Size::new(850, 800);
    const CYAN: [u8; 4] = [0, 217, 237, 255];
    let (mut desktop, _) = setup();
    desktop.appearance.theme = "walnut".into();
    desktop.appearance.mode = "dark".into();
    desktop.appearance.accent = "glow-cyan".into();
    desktop.sidebar.layout_width(SIZE.width as f32);
    let id = desktop.windows.library_id();
    let idle = paint(&desktop, id, SIZE);
    let _ = desktop.update(Message::Select(0));
    let playing = paint(&desktop, id, SIZE);
    let _ = desktop.update(Message::TogglePlayback);
    let paused = paint(&desktop, id, SIZE);
    let pixel = |frame: &[u8], x, y| {
        let offset = (y * SIZE.width as usize + x) * 4;
        <[u8; 4]>::try_from(&frame[offset..offset + 4]).unwrap()
    };

    assert_ne!(
        pixel(&idle, 92, 27),
        CYAN,
        "disabled control should remain muted"
    );
    for frame in [&playing, &paused] {
        assert_eq!(
            pixel(frame, 92, 27),
            CYAN,
            "active control needs the accent fill"
        );
        assert!(
            pixel(frame, 70, 40)[1] > pixel(&idle, 70, 40)[1] + 4,
            "the halo should extend beyond the button"
        );
    }
}

#[test]
fn it_draws_the_original_glow_around_the_seek_and_volume_rails() {
    const SIZE: Size<u32> = Size::new(850, 800);
    let (mut desktop, _) = setup();
    desktop.appearance.theme = "walnut".into();
    desktop.appearance.mode = "dark".into();
    desktop.sidebar.layout_width(SIZE.width as f32);
    let id = desktop.windows.library_id();
    let classic = paint(&desktop, id, SIZE);
    desktop.appearance.accent = "glow-cyan".into();
    let glow = paint(&desktop, id, SIZE);
    let at = |x, y| (y * SIZE.width as usize + x) * 4;
    for (x, y) in [(500, 59), (790, 37)] {
        let green = at(x, y) + 1;
        assert!(
            glow[green] > classic[green] + 4,
            "missing halo around the rail at ({x}, {y})"
        );
    }
    for y in [59, 62] {
        let green = at(500, y) + 1;
        let difference = glow[green].saturating_sub(classic[green]);
        assert!(
            difference <= 16,
            "the transport rail halo is too bright at {y}: {difference}"
        );
    }
    let volume_fill = at(790, 40);
    assert_eq!(&glow[volume_fill..volume_fill + 4], &[66, 233, 255, 255]);
}
