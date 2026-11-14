use iced::{
    Color, Event, Length, Rectangle, Renderer, Size, Theme,
    advanced::{
        Clipboard, Layout, Renderer as _, Shell, Widget, layout, mouse, renderer,
        widget::{self, Tree},
    },
    keyboard::{self, Key, key::Named},
    touch, window,
};

use super::Message;
use crate::desktop::sidebar::{Resize, SPLITTER_WIDTH};

const HIT_SLOP: f32 = 4.5;
const ACTIVE_LINE_WIDTH: f32 = 3.0;

pub(super) struct Divider {
    pub sidebar_width: f32,
    pub line: Color,
}

#[derive(Default)]
struct State {
    drag: Option<Drag>,
    hovered: bool,
}

#[derive(Clone, Copy)]
struct Drag {
    pointer: Pointer,
    start_x: f32,
    start_width: f32,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Pointer {
    Mouse,
    Touch(touch::Finger),
}

enum Gesture {
    Press(Pointer),
    Move(Pointer),
    Release(Pointer),
    Lose(Pointer),
    Cancel,
}

fn gesture(event: &Event) -> Option<Gesture> {
    use mouse::{Button, Event as Mouse};
    use touch::Event as Touch;
    Some(match event {
        Event::Mouse(Mouse::ButtonPressed(Button::Left)) => Gesture::Press(Pointer::Mouse),
        Event::Mouse(Mouse::CursorMoved { .. }) => Gesture::Move(Pointer::Mouse),
        Event::Mouse(Mouse::ButtonReleased(Button::Left)) => Gesture::Release(Pointer::Mouse),
        Event::Touch(Touch::FingerPressed { id, .. }) => Gesture::Press(Pointer::Touch(*id)),
        Event::Touch(Touch::FingerMoved { id, .. }) => Gesture::Move(Pointer::Touch(*id)),
        Event::Touch(Touch::FingerLifted { id, .. }) => Gesture::Release(Pointer::Touch(*id)),
        Event::Touch(Touch::FingerLost { id, .. }) => Gesture::Lose(Pointer::Touch(*id)),
        Event::Window(window::Event::Unfocused)
        | Event::Keyboard(keyboard::Event::KeyPressed {
            key: Key::Named(Named::Escape),
            ..
        }) => Gesture::Cancel,
        _ => return None,
    })
}

fn hit_area(bounds: Rectangle) -> Rectangle {
    Rectangle {
        x: bounds.x - HIT_SLOP,
        width: bounds.width + HIT_SLOP * 2.0,
        ..bounds
    }
}

impl Widget<Message, Theme, Renderer> for Divider {
    fn tag(&self) -> widget::tree::Tag {
        widget::tree::Tag::of::<State>()
    }

    fn state(&self) -> widget::tree::State {
        widget::tree::State::new(State::default())
    }

    fn size(&self) -> Size<Length> {
        Size::new(Length::Fixed(SPLITTER_WIDTH), Length::Fill)
    }

    fn layout(&mut self, _: &mut Tree, _: &Renderer, limits: &layout::Limits) -> layout::Node {
        layout::Node::new(limits.resolve(SPLITTER_WIDTH, Length::Fill, Size::ZERO))
    }

    fn update(
        &mut self,
        tree: &mut Tree,
        event: &Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        _: &Renderer,
        _: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
        _: &Rectangle,
    ) {
        let state = tree.state.downcast_mut::<State>();
        let hovered = cursor.is_over(hit_area(layout.bounds()));
        if state.hovered != hovered {
            state.hovered = hovered;
            shell.request_redraw();
        }
        let Some(gesture) = gesture(event) else {
            return;
        };
        let action = match (gesture, state.drag) {
            (Gesture::Press(pointer), None)
                if hovered && let Some(position) = cursor.position() =>
            {
                state.drag = Some(Drag {
                    pointer,
                    start_x: position.x,
                    start_width: self.sidebar_width,
                });
                None
            }
            (Gesture::Move(pointer), Some(drag))
                if drag.pointer == pointer
                    && let Some(position) = cursor.position() =>
            {
                Some(Resize::Preview(
                    drag.start_width + position.x - drag.start_x,
                ))
            }
            (Gesture::Release(pointer), Some(drag)) if drag.pointer == pointer => {
                state.drag = None;
                Some(Resize::Commit)
            }
            (Gesture::Lose(pointer), Some(drag)) if drag.pointer == pointer => {
                state.drag = None;
                Some(Resize::Cancel)
            }
            (Gesture::Cancel, Some(_)) => {
                state.drag = None;
                Some(Resize::Cancel)
            }
            _ => return,
        };
        if let Some(action) = action {
            shell.publish(Message::ResizeSidebar(action));
        }
        shell.capture_event();
        shell.request_redraw();
    }

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut Renderer,
        theme: &Theme,
        _: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        _: &Rectangle,
    ) {
        let bounds = layout.bounds();
        let state = tree.state.downcast_ref::<State>();
        let active = state.drag.is_some() || cursor.is_over(hit_area(bounds));
        let (width, color) = if active {
            (ACTIVE_LINE_WIDTH, theme.palette().primary)
        } else {
            (bounds.width, self.line)
        };
        renderer.fill_quad(
            renderer::Quad {
                bounds: Rectangle {
                    x: bounds.x + bounds.width - width,
                    width,
                    ..bounds
                },
                ..renderer::Quad::default()
            },
            color,
        );
    }

    fn mouse_interaction(
        &self,
        tree: &Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        _: &Rectangle,
        _: &Renderer,
    ) -> mouse::Interaction {
        if tree.state.downcast_ref::<State>().drag.is_some()
            || cursor.is_over(hit_area(layout.bounds()))
        {
            mouse::Interaction::ResizingHorizontally
        } else {
            mouse::Interaction::None
        }
    }
}
