use std::{collections::HashMap, ops::Range};

use iced::{
    Color, Font, Pixels, Rectangle, Renderer, Size, Vector,
    advanced::{
        Layout, layout,
        widget::{Id, Operation, Tree, operation::Scrollable},
    },
};

use crate::catalogue::IndexedTrack;

use super::{
    Desktop,
    cover_layout_tests::paint,
    queue_tests::{setup, snapshot},
    sidebar,
};

#[derive(Default)]
struct Table {
    labels: HashMap<String, Rectangle>,
    scroll: Option<(Rectangle, Rectangle)>,
}

impl Operation for Table {
    fn traverse(&mut self, visit: &mut dyn FnMut(&mut dyn Operation)) {
        visit(self);
    }

    fn text(&mut self, _: Option<&Id>, bounds: Rectangle, label: &str) {
        self.labels.insert(label.into(), bounds);
    }

    fn scrollable(
        &mut self,
        id: Option<&Id>,
        bounds: Rectangle,
        content: Rectangle,
        _: Vector,
        _: &mut dyn Scrollable,
    ) {
        if id == Some(&Id::new("indexed-songs")) {
            self.scroll = Some((bounds, content));
        }
    }
}

fn track_table(width: f32) -> Desktop {
    let (mut desktop, _) = setup();
    let mut sources = snapshot(&["records"]);
    sources.selected = Some("records".into());
    sources.tracks = [
        ("Restored track", 2, 11),
        ("Coast track", 1, 44),
        ("New track", 0, 45),
        ("Next track", 8, 55),
    ]
    .into_iter()
    .map(|(title, plays, duration_seconds)| IndexedTrack {
        id: title.into(),
        title: title.into(),
        artist: "Field Notes".into(),
        album: None,
        metadata_overridden: false,
        path: format!("/music/{title}.mp3").into(),
        plays,
        duration_seconds,
        library_id: "records".into(),
    })
    .collect();
    desktop.sources.snapshot = Some(sources);
    desktop.sidebar.layout_width(width);
    desktop
}

fn track_table_layout(desktop: &Desktop, width: f32) -> Table {
    let frame = Size::new(width, 800.0);
    let mut view = desktop.view(desktop.windows.library_id());
    let mut tree = Tree::new(&view);
    let renderer = Renderer::new(Font::with_name("Manrope"), Pixels(16.0));
    let widget = view.as_widget_mut();
    let node = widget.layout(&mut tree, &renderer, &layout::Limits::new(frame, frame));
    let mut table = Table::default();
    widget.operate(&mut tree, Layout::new(&node), &renderer, &mut table);
    table
}

#[test]
fn it_uses_the_original_track_title_columns_at_wide_and_compact_widths() {
    for (width, title_offset) in [(1200.0, 143.0), (850.0, 114.0)] {
        let desktop = track_table(width);
        let start = desktop.sidebar.visible_width() + sidebar::SPLITTER_WIDTH;
        let Table { labels, .. } = track_table_layout(&desktop, width);
        let first = labels["Restored track"];
        let second = labels["Coast track"];
        assert!((first.x - start - title_offset).abs() < 0.5, "{first:?}");
        assert!((second.y - first.y - 57.0).abs() < 0.5, "{second:?}");
    }
}

#[test]
fn it_shows_the_original_singular_and_plural_play_counts() {
    let Table { labels, .. } = track_table_layout(&track_table(1200.0), 1200.0);
    for count in ["0 plays", "1 play", "2 plays"] {
        assert!(labels.contains_key(count), "Missing {count:?}");
    }
}

#[test]
fn it_reserves_the_original_insets_before_and_after_the_track_list() {
    let mut desktop = track_table(1200.0);
    let tracks = &mut desktop.sources.snapshot.as_mut().unwrap().tracks;
    let extra = (0..12)
        .map(|index| IndexedTrack {
            title: format!("Additional track {index}"),
            ..tracks[0].clone()
        })
        .collect::<Vec<_>>();
    tracks.extend(extra);
    let Table { labels, scroll } = track_table_layout(&desktop, 1200.0);
    let (viewport, content) = scroll.expect("indexed tracks");
    let first = labels["Restored track"];
    let second_line_height = labels["Field Notes"].height;
    let metadata_height = first.height + 4.0 + second_line_height;
    let top_inset = first.y - viewport.y - (56.0 - metadata_height) / 2.0;
    assert!((top_inset - 8.0).abs() < 0.5, "top inset is {top_inset}");
    assert_eq!(content.height, 16.0 * 57.0 + 8.0 + 30.0);
}

fn pixel(frame: &[u8], width: usize, x: usize, y: usize) -> [u8; 4] {
    let offset = (y * width + x) * 4;
    frame[offset..offset + 4].try_into().unwrap()
}

fn assert_colour(actual: [u8; 4], base: Color, overlay: Color, opacity: f32) {
    for ((actual, base), overlay) in actual
        .into_iter()
        .zip(base.into_rgba8())
        .zip(overlay.into_rgba8())
    {
        let expected = f32::from(base) * (1.0 - opacity) + f32::from(overlay) * opacity;
        assert!(
            (f32::from(actual) - expected).abs() < 2.0,
            "{actual} vs {expected}"
        );
    }
}

#[test]
fn it_draws_the_original_alternating_fill_and_dividers() {
    let mut desktop = track_table(850.0);
    desktop.appearance.mode = "dark".into();
    let id = desktop.windows.library_id();
    let palette = desktop.theme(id).palette();
    let line = desktop.appearance.surfaces().line;
    let (viewport, _) = track_table_layout(&desktop, 850.0).scroll.unwrap();
    let first_row = viewport.y.round() as usize + 8;
    let frames = paint(&desktop, id, Size::new(850, 800));
    assert_colour(
        pixel(&frames, 850, 845, first_row + 56),
        palette.background,
        line,
        0.5,
    );
    assert_colour(
        pixel(&frames, 850, 845, first_row + 57 + 28),
        palette.background,
        palette.text,
        0.02,
    );
}

fn ink_width(frame: &[u8], columns: Range<usize>, middle: usize, background: [u8; 4]) -> usize {
    columns
        .filter(|&x| (middle - 11..middle + 11).any(|y| pixel(frame, 850, x, y) != background))
        .count()
}

#[test]
fn it_keeps_the_play_edit_and_playlist_glyphs_visible_in_compact_columns() {
    let desktop = track_table(850.0);
    let (viewport, _) = track_table_layout(&desktop, 850.0).scroll.unwrap();
    let middle = viewport.y.round() as usize + 8 + 28;
    let rendered = paint(&desktop, desktop.windows.library_id(), Size::new(850, 800));
    let background = pixel(&rendered, 850, 845, middle);
    let left = viewport.x.round() as usize + 15;
    for (columns, minimum_width) in [(left..left + 24, 9), (776..802, 11), (809..835, 11)] {
        assert!(
            ink_width(&rendered, columns.clone(), middle, background) >= minimum_width,
            "{columns:?}"
        );
    }
}

#[test]
fn it_keeps_compact_artwork_a_38_pixel_square() {
    let mut desktop = track_table(850.0);
    let artwork = [244, 17, 177, 255];
    let snapshot = desktop.sources.snapshot.as_mut().unwrap();
    let id = snapshot.tracks[0].id.clone();
    snapshot.covers.insert(
        id,
        iced::widget::image::Handle::from_rgba(64, 64, artwork.repeat(64 * 64)),
    );
    let (viewport, _) = track_table_layout(&desktop, 850.0).scroll.unwrap();
    let middle = viewport.y.round() as usize + 8 + 28;
    let rendered = paint(&desktop, desktop.windows.library_id(), Size::new(850, 800));
    assert_eq!(
        (0..850)
            .filter(|&x| {
                pixel(&rendered, 850, x, middle)
                    .iter()
                    .zip(artwork)
                    .all(|(actual, expected)| actual.abs_diff(expected) < 3)
            })
            .count(),
        36
    );
}

#[test]
fn it_aligns_different_digits_in_the_index_duration_and_history_columns() {
    iced::advanced::graphics::text::font_system()
        .write()
        .unwrap()
        .load_font(
            include_bytes!("../../assets/fonts/Manrope-Regular.ttf")
                .as_slice()
                .into(),
        );
    let Table { labels, .. } = track_table_layout(&track_table(1200.0), 1200.0);
    for (first, second) in [("1", "4"), ("0:11", "0:44"), ("2 plays", "8 plays")] {
        let a = labels[first];
        let b = labels[second];
        assert!(
            (a.width - b.width).abs() < 0.1,
            "{first} and {second}: {a:?}, {b:?}"
        );
        assert!(
            (a.x - b.x).abs() < 0.1,
            "{first} and {second}: {a:?}, {b:?}"
        );
    }
}
