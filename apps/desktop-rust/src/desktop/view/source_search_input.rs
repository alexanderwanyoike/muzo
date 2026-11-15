use iced::{
    Event, Length, Rectangle, Renderer, Size, Theme,
    advanced::{
        Clipboard, Layout, Shell, Widget, layout, mouse, renderer, text,
        widget::{Operation, Tree, tree},
    },
    keyboard::{Event::KeyPressed, Key, key::Named},
    widget::{TextInput, text_input::State},
};

use super::Message;

pub(super) struct SourceSearchInput<'a>(pub(super) TextInput<'a, Message>);

impl Widget<Message, Theme, Renderer> for SourceSearchInput<'_> {
    fn tag(&self) -> tree::Tag {
        self.0.tag()
    }

    fn state(&self) -> tree::State {
        self.0.state()
    }

    fn diff(&self, tree: &mut Tree) {
        self.0.diff(tree);
    }

    fn size(&self) -> Size<Length> {
        Widget::size(&self.0)
    }

    fn layout(
        &mut self,
        tree: &mut Tree,
        renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        Widget::layout(&mut self.0, tree, renderer, limits)
    }

    fn operate(
        &mut self,
        tree: &mut Tree,
        layout: Layout<'_>,
        renderer: &Renderer,
        operation: &mut dyn Operation,
    ) {
        self.0.operate(tree, layout, renderer, operation);
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
        let state = tree
            .state
            .downcast_ref::<State<<Renderer as text::Renderer>::Paragraph>>();
        if state.is_focused()
            && matches!(
                event,
                Event::Keyboard(KeyPressed {
                    key: Key::Named(Named::Escape),
                    ..
                })
            )
        {
            shell.publish(Message::ClearSourceSearch);
            shell.capture_event();
        } else {
            self.0.update(
                tree, event, layout, cursor, renderer, clipboard, shell, viewport,
            );
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
        Widget::draw(
            &self.0, tree, renderer, theme, style, layout, cursor, viewport,
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
        self.0
            .mouse_interaction(tree, layout, cursor, viewport, renderer)
    }
}
