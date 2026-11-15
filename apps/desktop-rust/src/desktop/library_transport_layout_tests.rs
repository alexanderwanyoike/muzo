use iced::Size;

use super::{cover_layout_tests::paint, queue_tests::setup};

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
