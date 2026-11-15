use iced::{
    Color, Element, Event, Length, Rectangle, Renderer, Size, Theme, Vector,
    advanced::{
        Clipboard, Layout, Renderer as _, Shell, Widget, layout, mouse, overlay, renderer,
        widget::{Operation, Tree, tree},
    },
};

use crate::appearance::Appearance;

use super::{Message, pulse_row_fills, selection_fill, tint};

#[derive(Clone, Copy)]
pub(super) struct Colors {
    base: Color,
    stripe: Color,
    hover: Color,
    current: Color,
}

impl Colors {
    pub(super) fn for_appearance(appearance: &Appearance) -> Self {
        const STRIPE_OPACITY: f32 = 0.02;
        const HOVER_OPACITY: f32 = 0.05;
        let palette = appearance.theme().palette();
        let hover = pulse_row_fills(appearance).map_or_else(
            || tint(palette.background, palette.text, HOVER_OPACITY),
            |fills| fills.hover,
        );
        Self {
            base: palette.background,
            stripe: tint(palette.background, palette.text, STRIPE_OPACITY),
            hover,
            current: selection_fill(appearance),
        }
    }
}

pub(super) struct TrackRow<'a> {
    content: Element<'a, Message>,
    background: Color,
    hover: Color,
}

#[derive(Default)]
struct State {
    hovered: bool,
}

impl<'a> TrackRow<'a> {
    pub(super) fn new(
        content: impl Into<Element<'a, Message>>,
        colors: Colors,
        current: bool,
        striped: bool,
    ) -> Self {
        let background = if current {
            colors.current
        } else if striped {
            colors.stripe
        } else {
            colors.base
        };
        Self {
            content: content.into(),
            background,
            hover: if current {
                colors.current
            } else {
                colors.hover
            },
        }
    }
}

impl Widget<Message, Theme, Renderer> for TrackRow<'_> {
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<State>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(State::default())
    }

    fn children(&self) -> Vec<Tree> {
        vec![Tree::new(&self.content)]
    }

    fn diff(&self, tree: &mut Tree) {
        tree.diff_children(&[self.content.as_widget()]);
    }

    fn size(&self) -> Size<Length> {
        self.content.as_widget().size()
    }

    fn layout(
        &mut self,
        tree: &mut Tree,
        renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        self.content
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
        self.content
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
        let state = tree.state.downcast_mut::<State>();
        let hovered = cursor.is_over(layout.bounds()) && cursor.is_over(*viewport);
        if std::mem::replace(&mut state.hovered, hovered) != hovered {
            shell.request_redraw();
        }
        self.content.as_widget_mut().update(
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
        let Some(bounds) = layout.bounds().intersection(viewport) else {
            return;
        };
        renderer.fill_quad(
            renderer::Quad {
                bounds,
                ..Default::default()
            },
            if cursor.is_over(bounds) {
                self.hover
            } else {
                self.background
            },
        );
        self.content.as_widget().draw(
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
        self.content.as_widget().mouse_interaction(
            &tree.children[0],
            layout,
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
        self.content.as_widget_mut().overlay(
            &mut tree.children[0],
            layout,
            renderer,
            viewport,
            translation,
        )
    }
}
