use iced::{
    Color, Element, Font, Pixels, Rectangle, Renderer, Size, Theme, Vector,
    advanced::{
        Layout, layout, mouse,
        renderer::{Headless, Style},
        widget::{Id, Operation, Tree, operation},
    },
};

use crate::{appearance::Appearance, catalogue::Playlist, desktop::queue_tests::snapshot};

use super::{Message, Sources, sidebar};

const TRACK_COUNT: u64 = 47;
const SCROLLBAR_WIDTH: f32 = 10.0;
const PANEL_SIZE: Size<u32> = Size::new(224, 250);
const PLAYLIST_ROWS: usize = 16;

#[derive(Default)]
struct SidebarBounds {
    viewport: Option<Rectangle>,
    track_count: Option<Rectangle>,
    count_label: String,
}

impl Operation for SidebarBounds {
    fn traverse(&mut self, visit: &mut dyn FnMut(&mut dyn Operation)) {
        visit(self);
    }

    fn scrollable(
        &mut self,
        _id: Option<&Id>,
        bounds: Rectangle,
        _content: Rectangle,
        _translation: Vector,
        _state: &mut dyn operation::Scrollable,
    ) {
        self.viewport = Some(bounds);
    }

    fn text(&mut self, _id: Option<&Id>, bounds: Rectangle, text: &str) {
        if text == self.count_label {
            self.track_count = Some(bounds);
        }
    }
}

fn layout_sidebar(
    panel: &mut Element<'_, Message>,
    tree: &mut Tree,
    renderer: &Renderer,
) -> (layout::Node, SidebarBounds) {
    let widget = panel.as_widget_mut();
    tree.diff(&*widget);
    let size = Size::new(PANEL_SIZE.width as f32, PANEL_SIZE.height as f32);
    let node = widget.layout(tree, renderer, &layout::Limits::new(size, size));
    let mut bounds = SidebarBounds {
        count_label: TRACK_COUNT.to_string(),
        ..SidebarBounds::default()
    };
    widget.operate(tree, Layout::new(&node), renderer, &mut bounds);
    (node, bounds)
}

fn space_after_count(sources: &Sources, tree: &mut Tree, renderer: &Renderer) -> f32 {
    let mut panel = sidebar(sources, Appearance::default().surfaces());
    let (_, bounds) = layout_sidebar(&mut panel, tree, renderer);
    let right = |rectangle: Rectangle| rectangle.x + rectangle.width;
    right(bounds.viewport.unwrap()) - right(bounds.track_count.unwrap())
}

fn playlists() -> Sources {
    let mut sources = Sources::new(None);
    let mut catalogue = snapshot(&[]);
    catalogue.playlists = (0..PLAYLIST_ROWS)
        .map(|index| Playlist {
            id: index.to_string(),
            name: format!("Playlist {index:02}"),
        })
        .collect();
    catalogue
        .playlist_counts
        .insert(catalogue.playlists[0].id.clone(), TRACK_COUNT);
    sources.snapshot = Some(catalogue);
    sources
}

#[test]
fn it_keeps_playlist_counts_clear_of_the_scrollbar_until_filtering_removes_it() {
    let mut sources = playlists();
    let renderer = Renderer::new(Font::DEFAULT, Pixels(16.0));
    let mut tree = Tree::empty();

    let overflow_gap = space_after_count(&sources, &mut tree, &renderer);
    sources.playlist_search = "Playlist 00".into();
    let filtered_gap = space_after_count(&sources, &mut tree, &renderer);

    assert!(
        overflow_gap >= SCROLLBAR_WIDTH,
        "overflow gap {overflow_gap}"
    );
    assert!(
        filtered_gap < SCROLLBAR_WIDTH,
        "filtered gap {filtered_gap}"
    );
}

#[test]
fn it_keeps_overlong_playlist_names_out_of_the_track_count() {
    let mut sources = playlists();
    let expected = pixels_beside_playlist_name(&sources);
    sources.snapshot.as_mut().unwrap().playlists[0].name =
        "An evening of field recordings from the headland and the east coast".into();

    assert!(
        pixels_beside_playlist_name(&sources) == expected,
        "the long name painted across the count or scrollbar"
    );
}

fn pixels_beside_playlist_name(sources: &Sources) -> Vec<u8> {
    let mut panel = sidebar(sources, Appearance::default().surfaces());
    let mut tree = Tree::empty();
    let mut renderer = Renderer::new(Font::DEFAULT, Pixels(16.0));
    let (node, bounds) = layout_sidebar(&mut panel, &mut tree, &renderer);
    panel.as_widget().draw(
        &tree,
        &mut renderer,
        &Theme::Light,
        &Style::default(),
        Layout::new(&node),
        mouse::Cursor::Unavailable,
        &Rectangle::with_size(node.size()),
    );
    let pixels = renderer.screenshot(PANEL_SIZE, 1.0, Color::WHITE);
    let count = bounds.track_count.unwrap();
    let row_start = count.y.floor() as usize;
    let row_end = (count.y + count.height).ceil() as usize;
    let pixels_after_name = count.x.floor() as usize * 4;
    pixels
        .chunks_exact(PANEL_SIZE.width as usize * 4)
        .take(row_end)
        .skip(row_start)
        .flat_map(|row| row[pixels_after_name..].iter().copied())
        .collect()
}
