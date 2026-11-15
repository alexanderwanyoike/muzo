use iced::{
    Event, Font, Pixels, Point, Rectangle, Renderer, Size,
    advanced::{
        Layout, Shell, clipboard, layout, mouse,
        renderer::{Headless, Style},
        widget::Tree,
    },
    time, window,
};
use iced_runtime::{UserInterface, user_interface::Cache};

use crate::appearance::sidebar::SidebarWidth;

use super::{
    Desktop, Message,
    cover_layout_tests::paint,
    library_sources::Operation,
    queue_tests::{setup, snapshot},
    transport_time_layout_tests::rendered_labels,
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

#[test]
fn it_draws_distinct_glyphs_for_dropbox_and_filesystem_sources() {
    let mut desktop = desktop_with_sources();
    let id = desktop.windows.library_id();
    let cloud = paint(&desktop, id, FRAME);
    desktop.sources.snapshot.as_mut().unwrap().libraries[0].kind = "filesystem".into();
    let folder = paint(&desktop, id, FRAME);

    assert!((372..388).any(|y| {
        let row = y * FRAME.width as usize * 4;
        let glyph = row + 17 * 4..row + 34 * 4;
        cloud[glyph.clone()] != folder[glyph]
    }));
}

#[test]
fn it_aligns_source_names_and_tabular_track_counts_to_the_original_columns() {
    load_fonts();
    let mut desktop = desktop_with_sources();
    let catalogue = desktop.sources.snapshot.as_mut().unwrap();
    catalogue.selected = None;
    catalogue.track_counts = [
        ("Selected library".into(), 11),
        ("Other library".into(), 48),
    ]
    .into();
    let labels = rendered_labels(
        &desktop,
        desktop.windows.library_id(),
        Size::new(FRAME.width as f32, FRAME.height as f32),
    );

    for (name, count) in [
        ("Selected library", "11 tracks"),
        ("Other library", "48 tracks"),
    ] {
        let title = labels[name];
        let tracks = labels[count];
        assert!((title.x - 39.0).abs() < 0.25, "{name}: {title:?}");
        assert!((title.height - 17.0).abs() < 0.25, "{name}: {title:?}");
        assert!((tracks.height - 13.0).abs() < 0.25, "{count}: {tracks:?}");
        assert!(
            (tracks.x + tracks.width - 274.0).abs() < 0.25,
            "{count}: {tracks:?}"
        );
    }
    assert!((labels["11 tracks"].width - labels["48 tracks"].width).abs() < 0.1);
}

#[test]
fn it_draws_source_name_and_count_ink_on_the_original_baselines() {
    load_fonts();
    let mut desktop = desktop_with_sources();
    desktop.appearance.theme = "walnut".into();
    desktop.appearance.mode = "dark".into();
    desktop.appearance.accent = "glow-cyan".into();
    let catalogue = desktop.sources.snapshot.as_mut().unwrap();
    catalogue.libraries[0].name = "Holloway Sessions".into();
    catalogue
        .track_counts
        .insert(catalogue.libraries[0].id.clone(), 2);
    let frame = paint_source_sidebar(&desktop, mouse::Cursor::Unavailable);
    let ink_rows = |range: std::ops::RangeInclusive<usize>| {
        let has_ink = |x: usize, y: usize| {
            let start = (y * FRAME.width as usize + x) * 4;
            frame[start..start + 3].iter().any(|channel| *channel > 130)
        };
        let rows: Vec<_> = (364..396)
            .filter(|&y| range.clone().any(|x| has_ink(x, y)))
            .collect();
        (*rows.first().unwrap(), *rows.last().unwrap())
    };

    assert_eq!(ink_rows(39..=219), (376, 387));
    assert_eq!(ink_rows(230..=274), (378, 383));
}

#[test]
fn it_uses_the_pulse_button_hover_fill_only_in_the_selection_area() {
    let mut desktop = desktop_with_sources();
    desktop.appearance.theme = "pulse".into();

    for (mode, hover, selected, panel) in [("light", 222, 216, 255), ("dark", 42, 42, 31)] {
        desktop.appearance.mode = mode.into();
        for (y, remove_fill) in [(FIRST_SOURCE_Y, selected), (SECOND_SOURCE_Y, panel)] {
            let cursor = mouse::Cursor::Available(Point::new(180.0, y as f32));
            let frame = paint_source_sidebar(&desktop, cursor);
            let at = |x: usize| {
                let start = (y * FRAME.width as usize + x) * 4;
                &frame[start..start + 4]
            };
            assert_eq!(at(180), [hover, hover, hover, 255], "Pulse {mode}");
            assert_eq!(
                at(298),
                [remove_fill, remove_fill, remove_fill, 255],
                "Pulse {mode} remove cell at y={y}"
            );
        }
    }
}

#[test]
fn it_preserves_the_original_gap_between_filtered_count_and_source_list() {
    const ACCENT: [u8; 4] = [0, 217, 237, 255];
    let mut desktop = desktop_with_sources();
    desktop.appearance.accent = "glow-cyan".into();
    desktop.sources.search = "Selected".into();
    let labels = rendered_labels(
        &desktop,
        desktop.windows.library_id(),
        (FRAME.width, FRAME.height).into(),
    );
    let count = labels["1 of 2 sources"];
    assert!((count.x - 18.0).abs() < 0.25, "{count:?}");
    assert!((count.y - 363.0).abs() < 0.5, "{count:?}");
    assert!((count.height - 14.0).abs() < 0.25, "{count:?}");

    let frame = paint_source_sidebar(&desktop, mouse::Cursor::Unavailable);
    let marked_rows: Vec<_> = (350..430)
        .filter(|&y| {
            let start = (y * FRAME.width as usize + 9) * 4;
            frame[start..start + 4] == ACCENT
        })
        .collect();
    assert_eq!(marked_rows, (385..417).collect::<Vec<_>>());
}

fn load_fonts() {
    let mut fonts = iced::advanced::graphics::text::font_system()
        .write()
        .unwrap();
    for font in crate::fonts::settings().fonts {
        fonts.load_font(font);
    }
}

fn paint_source_sidebar(desktop: &Desktop, cursor: mouse::Cursor) -> Vec<u8> {
    let id = desktop.windows.library_id();
    let mut renderer = Renderer::new(Font::with_name("Manrope"), Pixels(16.0));
    let mut ui = UserInterface::build(
        desktop.workspace(id),
        (FRAME.width, FRAME.height).into(),
        Cache::new(),
        &mut renderer,
    );
    ui.update(
        &[Event::Window(window::Event::RedrawRequested(
            time::Instant::now(),
        ))],
        cursor,
        &mut renderer,
        &mut clipboard::Null,
        &mut Vec::new(),
    );
    let theme = desktop.theme(id);
    ui.draw(&mut renderer, &theme, &Style::default(), cursor);
    renderer.screenshot(FRAME, 1.0, theme.palette().background)
}

fn click(desktop: &Desktop, point: Point) -> Vec<Message> {
    let mut view = desktop.workspace(desktop.windows.library_id());
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
