use iced::{
    Color, Element, Event, Length, Point, Rectangle, Renderer, Size, Theme, Vector,
    advanced::{
        Clipboard, Layout, Renderer as _, Shell, Widget, layout, mouse, overlay, renderer,
        widget::{self, Operation, Tree},
    },
    keyboard::{self, Key, key::Named},
    touch,
    widget::column,
    window,
};

use super::Message;
use crate::desktop::sidebar::{Resize, SPLITTER_WIDTH};

const HIT_SLOP: f32 = 4.5;
const ACTIVE_LINE_WIDTH: f32 = 3.0;

pub(super) struct Divider<'a> {
    body: Element<'a, Message>,
    sidebar_width: f32,
    line: Color,
    ink: Color,
}

#[derive(Default)]
struct State {
    drag: Option<Drag>,
    hovered: bool,
    mouse_position: Option<Point>,
    last_click: Option<mouse::Click>,
}

#[derive(Clone, Copy)]
struct Drag {
    pointer: Pointer,
    start_x: f32,
    last_position: Point,
    start_width: f32,
    moved: bool,
}

impl State {
    fn finish_drag(&mut self, drag: Drag, release: Option<Point>, area: Rectangle) -> Resize {
        self.drag = None;
        let previous = self.last_click.take();
        if drag.pointer == Pointer::Mouse
            && !drag.moved
            && let Some(position) = release.filter(|&position| area.contains(position))
        {
            let click = mouse::Click::new(position, mouse::Button::Left, previous);
            if click.kind() == mouse::click::Kind::Double {
                return Resize::Reset;
            }
            self.last_click = Some(click);
        }
        Resize::Commit
    }
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

impl<'a> Divider<'a> {
    pub(super) fn around(
        header: Element<'a, Message>,
        panes: Element<'a, Message>,
        footer: Element<'a, Message>,
        sidebar_width: f32,
        colors: (Color, Color),
    ) -> Self {
        let (line, ink) = colors;
        Self {
            body: column([header, panes, footer])
                .width(Length::Fill)
                .height(Length::Fill)
                .into(),
            sidebar_width,
            line,
            ink,
        }
    }

    fn handle_input(
        &self,
        state: &mut State,
        event: &Event,
        bounds: Rectangle,
        cursor: mouse::Cursor,
        shell: &mut Shell<'_, Message>,
    ) {
        if let Event::Window(window::Event::Resized(size)) = event {
            shell.publish(Message::ResizeSidebar(Resize::Viewport(size.width)));
            return;
        }
        let area = hit_area(bounds);
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
                state.last_click = None;
            }
            Event::Mouse(mouse::Event::ButtonPressed(button)) if *button != mouse::Button::Left => {
                state.last_click = None;
            }
            _ => {}
        }
        let mouse_position = state.mouse_position.or_else(|| cursor.position());
        let Some(gesture) = gesture(event, mouse_position) else {
            return;
        };
        let action = match (gesture, state.drag) {
            (Gesture::Press(pointer, position), None) => {
                if cursor.is_levitating() || !area.contains(position) {
                    state.last_click = None;
                    return;
                }
                state.drag = Some(Drag {
                    pointer,
                    start_x: position.x,
                    last_position: position,
                    start_width: self.sidebar_width,
                    moved: false,
                });
                None
            }
            (Gesture::Move(pointer, position), Some(drag)) if drag.pointer == pointer => {
                if drag.last_position != position {
                    state.drag = Some(Drag {
                        last_position: position,
                        moved: true,
                        ..drag
                    });
                }
                (drag.last_position.x != position.x).then_some(Resize::Preview(
                    drag.start_width + position.x - drag.start_x,
                ))
            }
            (Gesture::Release(pointer), Some(drag)) if drag.pointer == pointer => {
                Some(state.finish_drag(drag, mouse_position, area))
            }
            (Gesture::Lose(pointer), Some(drag)) if drag.pointer == pointer => {
                state.drag = None;
                state.last_click = None;
                Some(Resize::Cancel)
            }
            (Gesture::Cancel, _) => {
                state.last_click = None;
                if state.drag.take().is_none() {
                    return;
                }
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
}

fn regions(layout: Layout<'_>) -> (Layout<'_>, Layout<'_>) {
    let mut regions = layout.children();
    (
        regions.next().expect("library chrome"),
        regions.next().expect("sidebar divider"),
    )
}

fn divider_active(state: &State, bounds: Rectangle, cursor: mouse::Cursor) -> bool {
    state.drag.is_some() || cursor.is_over(hit_area(bounds))
}

impl Widget<Message, Theme, Renderer> for Divider<'_> {
    fn tag(&self) -> widget::tree::Tag {
        widget::tree::Tag::of::<State>()
    }

    fn state(&self) -> widget::tree::State {
        widget::tree::State::new(State::default())
    }

    fn children(&self) -> Vec<Tree> {
        vec![Tree::new(&self.body)]
    }

    fn diff(&self, tree: &mut Tree) {
        tree.diff_children(&[self.body.as_widget()]);
    }

    fn size(&self) -> Size<Length> {
        self.body.as_widget().size()
    }

    fn layout(
        &mut self,
        tree: &mut Tree,
        renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        let body = self
            .body
            .as_widget_mut()
            .layout(&mut tree.children[0], renderer, limits);
        let [_, panes, _] = body.children() else {
            unreachable!("library chrome contains a header, panes and footer");
        };
        let bounds = panes.bounds();
        let divider = layout::Node::new(Size::new(SPLITTER_WIDTH, bounds.height))
            .move_to(Point::new(bounds.x + self.sidebar_width, bounds.y));
        layout::Node::with_children(body.size(), vec![body, divider])
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
        let (body, divider) = regions(layout);
        let state = tree.state.downcast_mut::<State>();
        self.handle_input(state, event, divider.bounds(), cursor, shell);
        if !shell.is_event_captured() {
            let cursor = if divider_active(state, divider.bounds(), cursor) {
                cursor.levitate()
            } else {
                cursor
            };
            self.body.as_widget_mut().update(
                &mut tree.children[0],
                event,
                body,
                cursor,
                renderer,
                clipboard,
                shell,
                viewport,
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
        let (body, divider) = regions(layout);
        let bounds = divider.bounds();
        let state = tree.state.downcast_ref::<State>();
        let active = divider_active(state, bounds, cursor);
        self.body.as_widget().draw(
            &tree.children[0],
            renderer,
            theme,
            style,
            body,
            if active { cursor.levitate() } else { cursor },
            viewport,
        );
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
        viewport: &Rectangle,
        renderer: &Renderer,
    ) -> mouse::Interaction {
        let (body, divider) = regions(layout);
        if divider_active(tree.state.downcast_ref::<State>(), divider.bounds(), cursor) {
            mouse::Interaction::ResizingHorizontally
        } else {
            self.body.as_widget().mouse_interaction(
                &tree.children[0],
                body,
                cursor,
                viewport,
                renderer,
            )
        }
    }

    fn operate(
        &mut self,
        tree: &mut Tree,
        layout: Layout<'_>,
        renderer: &Renderer,
        operation: &mut dyn Operation,
    ) {
        self.body.as_widget_mut().operate(
            &mut tree.children[0],
            regions(layout).0,
            renderer,
            operation,
        );
    }

    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut Tree,
        layout: Layout<'b>,
        renderer: &Renderer,
        viewport: &Rectangle,
        translation: Vector,
    ) -> Option<overlay::Element<'b, Message, Theme, Renderer>> {
        self.body.as_widget_mut().overlay(
            &mut tree.children[0],
            regions(layout).0,
            renderer,
            viewport,
            translation,
        )
    }
}
