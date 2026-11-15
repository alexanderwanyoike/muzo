use iced::{
    Event, Font, Pixels, Point, Rectangle, Renderer, Size,
    advanced::{Layout, Shell, clipboard, layout, mouse, widget::Tree},
};

use crate::appearance::sidebar::SidebarWidth;

use super::{
    Desktop, Message,
    cover_layout_tests::paint,
    library_sources::Operation,
    queue_tests::{setup, snapshot},
};

const FRAME: Size<u32> = Size::new(850, 800);
const FIRST_SOURCE_Y: usize = 380;
const SECOND_SOURCE_Y: usize = FIRST_SOURCE_Y + 32;

fn desktop_with_sources() -> Desktop {
    let (mut desktop, _) = setup();
    let mut catalogue = snapshot(&["Selected library", "Other library"]);
    catalogue.selected = Some(catalogue.libraries[0].id.clone());
    desktop.sources.snapshot = Some(catalogue);
    desktop.sidebar.set(SidebarWidth::new(312.0));
    desktop.sidebar.layout_width(FRAME.width as f32);
    desktop
}

#[test]
fn it_draws_the_source_selection_across_the_whole_row_and_reserves_its_accent_edge() {
    const ACCENT: [u8; 4] = [0, 217, 237, 255];

    let mut desktop = desktop_with_sources();
    desktop.appearance.accent = "glow-cyan".into();

    for (material, mode, selected) in [
        ("walnut", "dark", [30, 53, 51, 255]),
        ("pulse", "light", [216, 216, 216, 255]),
        ("pulse", "dark", [42, 42, 42, 255]),
    ] {
        desktop.appearance.theme = material.into();
        desktop.appearance.mode = mode.into();
        let panel = desktop.appearance.surfaces().raised.into_rgba8();
        let pixels = paint(&desktop, desktop.windows.library_id(), FRAME);
        let at = |x, y| {
            let start = (y * FRAME.width as usize + x) * 4;
            &pixels[start..start + 4]
        };

        for x in [9, 10] {
            assert_eq!(
                at(x, FIRST_SOURCE_Y),
                ACCENT,
                "the selected {material} row's 2px accent edge"
            );
            assert_eq!(
                at(x, SECOND_SOURCE_Y),
                panel,
                "the unselected {material} row reserves an empty edge"
            );
        }
        for x in [11, 150, 300] {
            assert_eq!(
                at(x, FIRST_SOURCE_Y),
                selected,
                "the {material} selection at x={x}"
            );
        }
    }
}

#[test]
fn it_keeps_the_source_selection_and_remove_clicks_in_the_original_hit_areas() {
    let desktop = desktop_with_sources();

    for (y, name) in [
        (FIRST_SOURCE_Y, "Selected library"),
        (SECOND_SOURCE_Y, "Other library"),
    ] {
        let y = y as f32;
        let select = click(&desktop, Point::new(279.0, y));
        assert!(
            matches!(select.as_slice(), [Message::SourceOperation(Operation::Select(id))] if id == name),
            "the last pixel in the selection area should select {name}: {select:?}"
        );
        let remove = click(&desktop, Point::new(280.5, y));
        assert!(
            matches!(remove.as_slice(), [Message::RequestSourceRemoval(id)] if id == name),
            "the first pixel in the remove area should request removal of {name}: {remove:?}"
        );
        for outside in [y - 12.0, y + 12.0] {
            assert!(
                click(&desktop, Point::new(291.0, outside)).is_empty(),
                "clicking just above or below the remove button should leave {name} alone"
            );
        }
    }
}

fn click(desktop: &Desktop, point: Point) -> Vec<Message> {
    let mut view = desktop.view(desktop.windows.library_id());
    let mut tree = Tree::new(&view);
    let renderer = Renderer::new(Font::DEFAULT, Pixels(16.0));
    let bounds = Rectangle::with_size(Size::new(FRAME.width as f32, FRAME.height as f32));
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
            mouse::Cursor::Available(point),
            &renderer,
            &mut clipboard::Null,
            &mut Shell::new(&mut messages),
            &bounds,
        );
    }
    messages
}
