use std::collections::HashMap;

use iced::{
    Font, Pixels, Rectangle, Renderer, Size,
    advanced::{
        Layout, layout,
        widget::{Id, Operation, Tree},
    },
};

use crate::catalogue::IndexedTrack;

use super::{
    queue_tests::{setup, snapshot},
    sidebar,
};

#[derive(Default)]
struct Labels(HashMap<String, Rectangle>);

impl Operation for Labels {
    fn traverse(&mut self, visit: &mut dyn FnMut(&mut dyn Operation)) {
        visit(self);
    }

    fn text(&mut self, _: Option<&Id>, bounds: Rectangle, label: &str) {
        self.0.insert(label.into(), bounds);
    }
}

fn track_table_labels(width: f32) -> (Labels, f32) {
    let (mut desktop, _) = setup();
    let mut sources = snapshot(&["records"]);
    sources.selected = Some("records".into());
    sources.tracks = [
        ("Restored track", 2, 12),
        ("Coast track", 1, 30),
        ("New track", 0, 45),
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
    let content_start = desktop.sidebar.visible_width() + sidebar::SPLITTER_WIDTH;
    let frame = Size::new(width, 800.0);
    let mut view = desktop.view(desktop.windows.library_id());
    let mut tree = Tree::new(&view);
    let renderer = Renderer::new(Font::DEFAULT, Pixels(16.0));
    let widget = view.as_widget_mut();
    let node = widget.layout(&mut tree, &renderer, &layout::Limits::new(frame, frame));
    let mut labels = Labels::default();
    widget.operate(&mut tree, Layout::new(&node), &renderer, &mut labels);
    (labels, content_start)
}

#[test]
fn it_uses_the_original_track_title_columns_at_wide_and_compact_widths() {
    for (width, title_offset) in [(1200.0, 143.0), (850.0, 114.0)] {
        let (Labels(labels), start) = track_table_labels(width);
        let first = labels["Restored track"];
        let second = labels["Coast track"];
        assert!((first.x - start - title_offset).abs() < 0.5, "{first:?}");
        assert!((second.y - first.y - 57.0).abs() < 0.5, "{second:?}");
    }
}

#[test]
fn it_shows_the_original_singular_and_plural_play_counts() {
    let (Labels(labels), _) = track_table_labels(1200.0);
    for count in ["0 plays", "1 play", "2 plays"] {
        assert!(labels.contains_key(count), "Missing {count:?}");
    }
}
