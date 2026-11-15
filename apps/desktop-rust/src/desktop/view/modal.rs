use iced::{
    Element, Event, Length, Rectangle, Renderer, Size, Theme, Vector,
    advanced::{
        Clipboard, Layout, Shell, Widget, layout, mouse, overlay, renderer,
        renderer::Renderer as _,
        widget::{Operation, Tree},
    },
};

use super::Message;

type HandleKey = fn(&Event) -> Option<Message>;

pub(super) struct Modal<'a> {
    layers: Vec<Element<'a, Message>>,
    handle_key: Option<HandleKey>,
}

impl<'a> Modal<'a> {
    pub fn wrap(
        page: Element<'a, Message>,
        dialog: Option<(Element<'a, Message>, HandleKey)>,
    ) -> Element<'a, Message> {
        let mut layers = vec![page];
        let handle_key = dialog.map(|(dialog, handle_key)| {
            layers.push(dialog);
            handle_key
        });
        Element::new(Self { layers, handle_key })
    }

    fn active(&self) -> usize {
        self.layers.len() - 1
    }
}

impl Widget<Message, Theme, Renderer> for Modal<'_> {
    fn size(&self) -> Size<Length> {
        self.layers[0].as_widget().size()
    }

    fn children(&self) -> Vec<Tree> {
        self.layers.iter().map(Tree::new).collect()
    }

    fn diff(&self, tree: &mut Tree) {
        tree.diff_children(&self.layers);
    }

    fn layout(
        &mut self,
        tree: &mut Tree,
        renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        let page = self.layers[0]
            .as_widget_mut()
            .layout(&mut tree.children[0], renderer, limits);
        let size = page.size();
        let mut nodes = vec![page];
        if let Some(layer) = self.layers.get_mut(1) {
            nodes.push(layer.as_widget_mut().layout(
                &mut tree.children[1],
                renderer,
                &layout::Limits::new(size, size),
            ));
        }
        layout::Node::with_children(size, nodes)
    }

    fn update(
        &mut self,
        tree: &mut Tree,
        event: &Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
        viewport: &Rectangle,
    ) {
        let active = self.active();
        self.layers[active].as_widget_mut().update(
            &mut tree.children[active],
            event,
            layout.child(active),
            cursor,
            renderer,
            clipboard,
            shell,
            viewport,
        );
        if let Some(handle_key) = &self.handle_key {
            if let Some(message) = handle_key(event) {
                shell.publish(message);
            }
            if !matches!(event, Event::Window(_)) {
                shell.capture_event();
            }
        }
    }

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut Renderer,
        theme: &Theme,
        style: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        let active = self.active();
        let draw = |renderer: &mut Renderer, index: usize, cursor| {
            self.layers[index].as_widget().draw(
                &tree.children[index],
                renderer,
                theme,
                style,
                layout.child(index),
                cursor,
                viewport,
            );
        };
        draw(
            renderer,
            0,
            if active == 0 {
                cursor
            } else {
                mouse::Cursor::Unavailable
            },
        );
        if active != 0 {
            renderer.with_layer(layout.bounds(), |renderer| draw(renderer, active, cursor));
        }
    }

    fn operate(
        &mut self,
        tree: &mut Tree,
        layout: Layout<'_>,
        renderer: &Renderer,
        operation: &mut dyn Operation,
    ) {
        let active = self.active();
        self.layers[active].as_widget_mut().operate(
            &mut tree.children[active],
            layout.child(active),
            renderer,
            operation,
        );
    }

    fn mouse_interaction(
        &self,
        tree: &Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
        renderer: &Renderer,
    ) -> mouse::Interaction {
        let active = self.active();
        self.layers[active].as_widget().mouse_interaction(
            &tree.children[active],
            layout.child(active),
            cursor,
            viewport,
            renderer,
        )
    }

    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut Tree,
        layout: Layout<'b>,
        renderer: &Renderer,
        viewport: &Rectangle,
        translation: Vector,
    ) -> Option<overlay::Element<'b, Message, Theme, Renderer>> {
        let active = self.active();
        self.layers[active].as_widget_mut().overlay(
            &mut tree.children[active],
            layout.child(active),
            renderer,
            viewport,
            translation,
        )
    }
}
