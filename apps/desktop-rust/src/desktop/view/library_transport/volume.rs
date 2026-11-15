use iced::{
    Element, Event, Length, Rectangle, Renderer, Size, Theme,
    advanced::{
        Clipboard, Layout, Shell, Widget, layout, mouse, renderer,
        widget::{Tree, tree},
    },
    widget::Slider,
};

use super::{Message, compact::VOLUME_THUMB_RADIUS};

type Control = Slider<'static, f32, Message, Theme>;

pub(super) fn within_thumb_travel(slider: Control) -> Element<'static, Message> {
    Element::new(VolumeSlider(slider))
}

pub(super) fn thumb_travel(width: f32) -> f32 {
    (width - VOLUME_THUMB_RADIUS * 2.0).max(0.0)
}

fn volume_cursor(cursor: mouse::Cursor, bounds: Rectangle) -> mouse::Cursor {
    let Some(mut point) = cursor.position() else {
        return cursor;
    };
    let right = bounds.x + bounds.width;
    let travel = thumb_travel(bounds.width);
    if point.x < bounds.x || point.x >= right || travel == 0.0 {
        return cursor;
    }

    // Chromium maps pointer position between the extreme thumb centers. Iced maps the whole track.
    let fraction = ((point.x - bounds.x - VOLUME_THUMB_RADIUS) / travel).clamp(0.0, 1.0);
    point.x = (bounds.x + fraction * bounds.width).min(right.next_down());
    mouse::Cursor::Available(point)
}

struct VolumeSlider(Control);

impl Widget<Message, Theme, Renderer> for VolumeSlider {
    fn tag(&self) -> tree::Tag {
        <Control as Widget<Message, Theme, Renderer>>::tag(&self.0)
    }

    fn state(&self) -> tree::State {
        <Control as Widget<Message, Theme, Renderer>>::state(&self.0)
    }

    fn size(&self) -> Size<Length> {
        <Control as Widget<Message, Theme, Renderer>>::size(&self.0)
    }

    fn layout(
        &mut self,
        tree: &mut Tree,
        renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        self.0.layout(tree, renderer, limits)
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
        self.0.update(
            tree,
            event,
            layout,
            volume_cursor(cursor, layout.bounds()),
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
        self.0
            .draw(tree, renderer, theme, style, layout, cursor, viewport);
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
