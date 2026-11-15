use iced::{
    Element, Font, Pixels, Rectangle, Renderer, Size,
    advanced::{
        Layout, layout,
        widget::{Id, Operation, Tree},
    },
};

use crate::{appearance::Appearance, catalogue::Playlist, desktop::queue_tests::snapshot};

use super::{Message, Sources, playlists, sources};

const NARROW_PANEL: f32 = 176.0;
const WIDE_PANEL: f32 = 344.0;
const PANEL_HEIGHT: f32 = 250.0;
const LONG_NAME: &str = "Field recordings from the headland";
const SOURCE_ID: &str = "recordings";
const PLAYLIST_ID: &str = "collection";
const TRACK_COUNT: u64 = 47;

#[derive(Default)]
struct Labels(Vec<String>);

impl Operation for Labels {
    fn traverse(&mut self, visit: &mut dyn FnMut(&mut dyn Operation)) {
        visit(self);
    }

    fn text(&mut self, _id: Option<&Id>, _bounds: Rectangle, value: &str) {
        self.0.push(value.into());
    }
}

fn rendered_labels(panel: &mut Element<'_, Message>, tree: &mut Tree, width: f32) -> Vec<String> {
    let renderer = Renderer::new(Font::DEFAULT, Pixels(16.0));
    let widget = panel.as_widget_mut();
    tree.diff(&*widget);
    let size = Size::new(width, PANEL_HEIGHT);
    let node = widget.layout(tree, &renderer, &layout::Limits::new(size, size));
    let mut labels = Labels::default();
    widget.operate(tree, Layout::new(&node), &renderer, &mut labels);
    labels.0
}

fn named_rows(name: &str) -> Sources {
    let mut catalogue = snapshot(&[SOURCE_ID]);
    catalogue.libraries[0].name = name.into();
    catalogue.track_counts.insert(SOURCE_ID.into(), TRACK_COUNT);
    catalogue.playlists.push(Playlist {
        id: PLAYLIST_ID.into(),
        name: name.into(),
    });
    catalogue
        .playlist_counts
        .insert(PLAYLIST_ID.into(), TRACK_COUNT);
    let mut sources = Sources::new(None);
    sources.snapshot = Some(catalogue);
    sources
}

#[test]
fn it_ellipsizes_both_sidebar_names_and_shows_the_full_names_when_widened() {
    let records = named_rows(LONG_NAME);
    let appearance = Appearance::default();

    for (mut panel, count) in [
        (
            sources::sidebar(&records, &appearance),
            format!("{TRACK_COUNT} tracks"),
        ),
        (
            playlists::sidebar(&records, appearance.surfaces()),
            TRACK_COUNT.to_string(),
        ),
    ] {
        let mut tree = Tree::empty();
        let narrow = rendered_labels(&mut panel, &mut tree, NARROW_PANEL);
        let wide = rendered_labels(&mut panel, &mut tree, WIDE_PANEL);

        let prefix = narrow.iter().find_map(|label| label.strip_suffix('…'));
        assert!(
            prefix.is_some_and(|prefix| !prefix.is_empty() && LONG_NAME.starts_with(prefix)),
            "the narrow row had no abbreviated name: {narrow:?}"
        );
        assert!(narrow.contains(&count));
        assert!(wide.iter().any(|label| label == LONG_NAME));
        assert!(wide.contains(&count));
    }
}

#[test]
fn it_ellipsizes_before_a_whole_combining_character() {
    const CLUSTER: &str = "e\u{301}";
    let name = CLUSTER.repeat(42);
    let records = named_rows(&name);
    let mut panel = playlists::sidebar(&records, Appearance::default().surfaces());
    let labels = rendered_labels(&mut panel, &mut Tree::empty(), NARROW_PANEL);

    let prefix = labels.iter().find_map(|label| label.strip_suffix('…'));
    assert!(
        prefix.is_some_and(|prefix| !prefix.is_empty() && prefix.replace(CLUSTER, "").is_empty()),
        "the label should end at a grapheme boundary: {labels:?}"
    );
}

#[test]
fn it_renders_sidebar_names_on_one_line_when_saved_names_contain_whitespace() {
    let records = named_rows(" Field\t recordings\r\nfrom  the headland ");
    let appearance = Appearance::default();

    for mut panel in [
        sources::sidebar(&records, &appearance),
        playlists::sidebar(&records, appearance.surfaces()),
    ] {
        let labels = rendered_labels(&mut panel, &mut Tree::empty(), WIDE_PANEL);
        assert!(
            labels.iter().any(|label| label == LONG_NAME),
            "the saved whitespace should fit on one line: {labels:?}"
        );
    }
}
