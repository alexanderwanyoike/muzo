use iced::{
    Color, Event, Length, Point, Rectangle, Renderer, Size, Theme,
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
    pub ink: Color,
}

#[derive(Default)]
struct State {
    drag: Option<Drag>,
    hovered: bool,
    mouse_position: Option<Point>,
}

#[derive(Clone, Copy)]
struct Drag {
    pointer: Pointer,
    start_x: f32,
    last_x: f32,
    start_width: f32,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Pointer {
    Mouse,
    Touch(touch::Finger),
}

enum Gesture {
    Press(Pointer, Point),
    Move(Pointer, Point),
    Release(Pointer),
    Lose(Pointer),
    Cancel,
}

fn gesture(event: &Event, mouse_position: Option<Point>) -> Option<Gesture> {
    use mouse::{Button, Event as Mouse};
    use touch::Event as Touch;
    Some(match event {
        Event::Mouse(Mouse::ButtonPressed(Button::Left)) => {
            Gesture::Press(Pointer::Mouse, mouse_position?)
        }
        Event::Mouse(Mouse::CursorMoved { position }) => Gesture::Move(Pointer::Mouse, *position),
        Event::Mouse(Mouse::ButtonReleased(Button::Left)) => Gesture::Release(Pointer::Mouse),
        Event::Touch(Touch::FingerPressed { id, position }) => {
            Gesture::Press(Pointer::Touch(*id), *position)
        }
        Event::Touch(Touch::FingerMoved { id, position }) => {
            Gesture::Move(Pointer::Touch(*id), *position)
        }
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
        if let Event::Window(window::Event::Resized(size)) = event {
            shell.publish(Message::ResizeSidebar(Resize::Viewport(size.width)));
            return;
        }
        let state = tree.state.downcast_mut::<State>();
        let area = hit_area(layout.bounds());
        let hovered = cursor.is_over(area);
        if state.hovered != hovered {
            state.hovered = hovered;
            shell.request_redraw();
        }
        // Iced batches input events with the latest cursor. A mouse press has no position.
        match event {
            Event::Mouse(mouse::Event::CursorMoved { position }) => {
                state.mouse_position = Some(*position);
            }
            Event::Mouse(mouse::Event::CursorLeft) | Event::Window(window::Event::Unfocused) => {
                state.mouse_position = None;
            }
            _ => {}
        }
        let Some(gesture) = gesture(event, state.mouse_position.or_else(|| cursor.position()))
        else {
            return;
        };
        let action = match (gesture, state.drag) {
            (Gesture::Press(pointer, position), None)
                if !cursor.is_levitating() && area.contains(position) =>
            {
                state.drag = Some(Drag {
                    pointer,
                    start_x: position.x,
                    last_x: position.x,
                    start_width: self.sidebar_width,
                });
                None
            }
            (Gesture::Move(pointer, position), Some(drag)) if drag.pointer == pointer => {
                if drag.last_x == position.x {
                    None
                } else {
                    state.drag = Some(Drag {
                        last_x: position.x,
                        ..drag
                    });
                    Some(Resize::Preview(
                        drag.start_width + position.x - drag.start_x,
                    ))
                }
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
        _: &Theme,
        _: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        _: &Rectangle,
    ) {
        let bounds = layout.bounds();
        let state = tree.state.downcast_ref::<State>();
        let active = state.drag.is_some() || cursor.is_over(hit_area(bounds));
        let (width, color) = if active {
            (ACTIVE_LINE_WIDTH, self.ink)
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
