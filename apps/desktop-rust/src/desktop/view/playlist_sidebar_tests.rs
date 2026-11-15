use iced::{
    Font, Pixels, Rectangle, Renderer, Size, Vector,
    advanced::{
        Layout, layout,
        widget::{Id, Operation, Tree, operation},
    },
};

use crate::{appearance::Appearance, catalogue::Playlist, desktop::queue_tests::snapshot};

use super::{Sources, sidebar};

const TRACK_COUNT: &str = "47";
const SCROLLBAR_WIDTH: f32 = 10.0;

#[derive(Default)]
struct SidebarBounds {
    viewport: Option<Rectangle>,
    track_count: Option<Rectangle>,
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
        if text == TRACK_COUNT {
            self.track_count = Some(bounds);
        }
    }
}

fn space_after_count(sources: &Sources, tree: &mut Tree, renderer: &Renderer) -> f32 {
    let mut panel = sidebar(sources, Appearance::default().surfaces());
    let widget = panel.as_widget_mut();
    tree.diff(&*widget);
    let size = Size::new(224.0, 250.0);
    let node = widget.layout(tree, renderer, &layout::Limits::new(size, size));
    let mut bounds = SidebarBounds::default();
    widget.operate(tree, Layout::new(&node), renderer, &mut bounds);
    let right = |rectangle: Rectangle| rectangle.x + rectangle.width;
    right(bounds.viewport.unwrap()) - right(bounds.track_count.unwrap())
}

#[test]
fn it_keeps_playlist_counts_clear_of_the_scrollbar_until_filtering_removes_it() {
    let mut sources = Sources::new(None);
    let mut catalogue = snapshot(&[]);
    catalogue.playlists = (0..16)
        .map(|index| Playlist {
            id: index.to_string(),
            name: format!("Playlist {index:02}"),
        })
        .collect();
    catalogue.playlist_counts.insert("0".into(), 47);
    sources.snapshot = Some(catalogue);
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
