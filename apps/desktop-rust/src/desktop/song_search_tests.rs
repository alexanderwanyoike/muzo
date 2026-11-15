use iced::{
    Font, Pixels, Rectangle, Renderer, Size,
    advanced::{
        Layout, layout,
        widget::{Id, Operation, Tree, operation::TextInput},
    },
};

use super::{Desktop, queue_tests::setup};

#[derive(Default)]
struct SearchInputs(Vec<Id>);

impl Operation for SearchInputs {
    fn traverse(&mut self, visit: &mut dyn FnMut(&mut dyn Operation)) {
        visit(self);
    }

    fn text_input(&mut self, id: Option<&Id>, _: Rectangle, _: &mut dyn TextInput) {
        self.0.extend(id.cloned());
    }
}

fn search_inputs(desktop: &Desktop) -> Vec<Id> {
    let mut view = desktop.workspace(desktop.windows.library_id());
    let mut tree = Tree::new(&view);
    let renderer = Renderer::new(Font::DEFAULT, Pixels(16.0));
    let frame = Size::new(1000.0, 800.0);
    let widget = view.as_widget_mut();
    let node = widget.layout(&mut tree, &renderer, &layout::Limits::new(frame, frame));
    let mut inputs = SearchInputs::default();
    widget.operate(&mut tree, Layout::new(&node), &renderer, &mut inputs);
    inputs.0
}

#[test]
fn it_offers_global_song_search_even_without_a_selected_source() {
    let (desktop, _) = setup();

    assert!(
        search_inputs(&desktop).contains(&Id::new("track-search")),
        "Songs must offer search before the user selects a source"
    );
}
