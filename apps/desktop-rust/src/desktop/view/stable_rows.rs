use std::collections::HashMap;

use iced::{
    Element, Event, Length, Padding, Rectangle, Renderer, Size, Theme, Vector,
    advanced::{
        Clipboard, Layout, Shell, Widget, layout, mouse, overlay, renderer,
        widget::{Operation, Tree, tree},
    },
    widget::column,
};

use super::Message;

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub(super) enum Key {
    Track(usize),
    Gap(usize),
    Notice,
}

pub(super) struct StableRows<'a> {
    keys: Vec<Key>,
    column: Element<'a, Message>,
}

impl<'a> StableRows<'a> {
    pub(super) fn new(rows: Vec<(Key, Element<'a, Message>)>, padding: Padding) -> Self {
        let (keys, rows): (Vec<_>, Vec<_>) = rows.into_iter().unzip();
        Self {
            keys,
            column: column(rows).padding(padding).into(),
        }
    }
}

impl Widget<Message, Theme, Renderer> for StableRows<'_> {
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<Vec<Key>>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(self.keys.clone())
    }

    fn children(&self) -> Vec<Tree> {
        vec![Tree::new(&self.column)]
    }

    fn diff(&self, tree: &mut Tree) {
        let keys = tree.state.downcast_mut::<Vec<Key>>();
        let column = &mut tree.children[0];
        if *keys != self.keys {
            // Iced reconciles equal-length arrays by position. Reorder the retained
            // trees first so an editor keeps its cursor and focus when its row moves.
            let mut retained: HashMap<_, _> = keys
                .iter()
                .copied()
                .zip(std::mem::take(&mut column.children))
                .collect();
            column.children = self
                .keys
                .iter()
                .zip(self.column.as_widget().children())
                .map(|(key, fresh)| retained.remove(key).unwrap_or(fresh))
                .collect();
            keys.clone_from(&self.keys);
        }
        column.diff(self.column.as_widget());
    }

    fn size(&self) -> Size<Length> {
        self.column.as_widget().size()
    }

    fn layout(
        &mut self,
        tree: &mut Tree,
        renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        self.column
            .as_widget_mut()
            .layout(&mut tree.children[0], renderer, limits)
    }

    fn operate(
        &mut self,
        tree: &mut Tree,
        layout: Layout<'_>,
        renderer: &Renderer,
        operation: &mut dyn Operation,
    ) {
        self.column
            .as_widget_mut()
            .operate(&mut tree.children[0], layout, renderer, operation);
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
        self.column.as_widget_mut().update(
            &mut tree.children[0],
            event,
            layout,
            cursor,
            renderer,
            clipboard,
            shell,
            viewport,
        );
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
        self.column.as_widget().draw(
            &tree.children[0],
            renderer,
            theme,
            style,
            layout,
            cursor,
            viewport,
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
        self.column.as_widget().mouse_interaction(
            &tree.children[0],
            layout,
            cursor,
            viewport,
            renderer,
        )
    }

    fn overlay<'a>(
        &'a mut self,
        tree: &'a mut Tree,
        layout: Layout<'a>,
        renderer: &Renderer,
        viewport: &Rectangle,
        translation: Vector,
    ) -> Option<overlay::Element<'a, Message, Theme, Renderer>> {
        self.column.as_widget_mut().overlay(
            &mut tree.children[0],
            layout,
            renderer,
            viewport,
            translation,
        )
    }
}
