use iced::{
    Element, Event, Length, Point, Rectangle, Renderer, Size, Theme, Vector,
    advanced::{
        Clipboard, Layout, Shell, Widget, layout, mouse, renderer,
        widget::{Tree, tree},
    },
    widget::Slider,
    window,
};

use super::{Message, compact::VOLUME_THUMB_RADIUS};

type Control = Slider<'static, f32, Message, Theme>;

pub(super) const VOLUME_STEP: f32 = 0.05;

pub(super) fn within_thumb_travel(slider: Control, level: f32) -> Element<'static, Message> {
    Element::new(VolumeSlider {
        slider: slider.into(),
        level,
    })
}

pub(super) fn thumb_travel(width: f32) -> f32 {
    (width - VOLUME_THUMB_RADIUS * 2.0).max(0.0)
}

fn volume_cursor(cursor: mouse::Cursor, bounds: Rectangle) -> mouse::Cursor {
    let Some(point) = cursor.land().position() else {
        return cursor;
    };
    let right = bounds.x + bounds.width;
    let travel = thumb_travel(bounds.width);
    if point.x < bounds.x || point.x >= right || travel == 0.0 {
        return cursor;
    }

    // Chromium maps pointer position between the extreme thumb centers. Iced maps the whole track.
    let fraction = volume_fraction(point.x, bounds, travel);
    let x = (bounds.x + fraction * bounds.width).min(right.next_down());
    cursor + Vector::new(x - point.x, 0.0)
}

fn volume_fraction(x: f32, bounds: Rectangle, travel: f32) -> f32 {
    ((x - bounds.x - VOLUME_THUMB_RADIUS) / travel).clamp(0.0, 1.0)
}

struct VolumeSlider {
    slider: Element<'static, Message>,
    level: f32,
}

#[derive(Default)]
struct Drag {
    thumb: bool,
}

impl VolumeSlider {
    fn thumb_bounds(&self, bounds: Rectangle) -> Rectangle {
        let diameter = VOLUME_THUMB_RADIUS * 2.0;
        Rectangle::new(
            Point::new(
                bounds.x + thumb_travel(bounds.width) * self.level,
                bounds.center().y - VOLUME_THUMB_RADIUS,
            ),
            Size::new(diameter, diameter),
        )
    }

    fn handle_thumb_drag(
        &mut self,
        drag: &mut Drag,
        event: &Event,
        bounds: Rectangle,
        cursor: mouse::Cursor,
        shell: &mut Shell<'_, Message>,
    ) -> bool {
        match event {
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left))
                if cursor.is_over(self.thumb_bounds(bounds)) =>
            {
                // The original thumb starts dragging without changing the level on mouse-down.
                drag.thumb = true;
                shell.capture_event();
                true
            }
            Event::Mouse(mouse::Event::CursorMoved { position }) if drag.thumb => {
                let travel = thumb_travel(bounds.width);
                if travel > 0.0 {
                    let fraction = volume_fraction(position.x, bounds, travel);
                    let level = (fraction / VOLUME_STEP).round() * VOLUME_STEP;
                    if self.level != level {
                        self.level = level;
                        shell.publish(Message::SetVolume(level));
                    }
                }
                shell.capture_event();
                true
            }
            Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left))
            | Event::Window(window::Event::Unfocused) => std::mem::take(&mut drag.thumb),
            _ => false,
        }
    }
}

impl Widget<Message, Theme, Renderer> for VolumeSlider {
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<Drag>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(Drag::default())
    }

    fn children(&self) -> Vec<Tree> {
        vec![Tree::new(&self.slider)]
    }

    fn diff(&self, tree: &mut Tree) {
        tree.diff_children(&[self.slider.as_widget()]);
    }

    fn size(&self) -> Size<Length> {
        self.slider.as_widget().size()
    }

    fn layout(
        &mut self,
        tree: &mut Tree,
        renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        self.slider
            .as_widget_mut()
            .layout(&mut tree.children[0], renderer, limits)
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
        let bounds = layout.bounds();
        if self.handle_thumb_drag(
            tree.state.downcast_mut::<Drag>(),
            event,
            bounds,
            cursor,
            shell,
        ) {
            return;
        }
        self.slider.as_widget_mut().update(
            &mut tree.children[0],
            event,
            layout,
            volume_cursor(cursor, bounds),
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
        self.slider.as_widget().draw(
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
        self.slider.as_widget().mouse_interaction(
            &tree.children[0],
            layout,
            cursor,
            viewport,
            renderer,
        )
    }
}
