use iced::{
    Element, Event, Length, Point, Rectangle, Renderer, Size, Theme, Vector,
    advanced::{
        Clipboard, Layout, Overlay, Shell, Widget, layout, mouse, overlay, renderer,
        widget::{Operation, Tree},
    },
};

use super::Message;

pub(super) struct Choice<'a> {
    pub anchor: Element<'a, Message>,
    pub popup: Option<Element<'a, Message>>,
    pub enabled: bool,
}

impl Widget<Message, Theme, Renderer> for Choice<'_> {
    fn children(&self) -> Vec<Tree> {
        std::iter::once(Tree::new(&self.anchor))
            .chain(self.popup.iter().map(Tree::new))
            .collect()
    }

    fn diff(&self, tree: &mut Tree) {
        if let Some(popup) = &self.popup {
            tree.diff_children(&[self.anchor.as_widget(), popup.as_widget()]);
        } else {
            tree.diff_children(&[self.anchor.as_widget()]);
        }
    }

    fn size(&self) -> Size<Length> {
        self.anchor.as_widget().size()
    }

    fn layout(
        &mut self,
        tree: &mut Tree,
        renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        self.anchor
            .as_widget_mut()
            .layout(&mut tree.children[0], renderer, limits)
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
        self.anchor.as_widget().draw(
            &tree.children[0],
            renderer,
            theme,
            style,
            layout,
            cursor,
            viewport,
        );
    }

    fn update(
        &mut self,
        _: &mut Tree,
        event: &Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        _: &Renderer,
        _: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
        viewport: &Rectangle,
    ) {
        if self.enabled
            && press_in(event, cursor, layout.bounds())
            && press_in(event, cursor, *viewport)
        {
            shell.publish(Message::ToggleTrackPlaylistPopup);
            shell.capture_event();
        }
    }

    fn mouse_interaction(
        &self,
        _: &Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
        _: &Renderer,
    ) -> mouse::Interaction {
        if self.enabled && cursor.is_over(layout.bounds()) && cursor.is_over(*viewport) {
            mouse::Interaction::Pointer
        } else {
            mouse::Interaction::None
        }
    }

    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut Tree,
        layout: Layout<'b>,
        _: &Renderer,
        _: &Rectangle,
        translation: Vector,
    ) -> Option<overlay::Element<'b, Message, Theme, Renderer>> {
        let popup = self.popup.as_mut()?;
        Some(overlay::Element::new(Box::new(Popup {
            content: popup,
            tree: &mut tree.children[1],
            anchor: layout.bounds() + translation,
        })))
    }
}

struct Popup<'a, 'b> {
    content: &'a mut Element<'b, Message>,
    tree: &'a mut Tree,
    anchor: Rectangle,
}

impl Overlay<Message, Theme, Renderer> for Popup<'_, '_> {
    fn layout(&mut self, renderer: &Renderer, bounds: Size) -> layout::Node {
        let below = (bounds.height - self.anchor.y - self.anchor.height).max(0.0);
        let above = self.anchor.y.clamp(0.0, bounds.height);
        let width = self.anchor.width.min(bounds.width);
        let limits =
            layout::Limits::new(Size::ZERO, Size::new(width, below.max(above))).width(width);
        let node = self
            .content
            .as_widget_mut()
            .layout(self.tree, renderer, &limits);
        let x = self.anchor.x.clamp(0.0, bounds.width - width);
        let y = if below >= above {
            self.anchor.y + self.anchor.height
        } else {
            self.anchor.y - node.size().height
        };
        let max_y = (bounds.height - node.size().height).max(0.0);
        node.move_to(Point::new(x, y.clamp(0.0, max_y)))
    }

    fn draw(
        &self,
        renderer: &mut Renderer,
        theme: &Theme,
        style: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
    ) {
        self.content.as_widget().draw(
            self.tree,
            renderer,
            theme,
            style,
            layout,
            cursor,
            &layout.bounds(),
        );
    }

    fn operate(&mut self, layout: Layout<'_>, renderer: &Renderer, operation: &mut dyn Operation) {
        self.content
            .as_widget_mut()
            .operate(self.tree, layout, renderer, operation);
    }

    fn update(
        &mut self,
        event: &Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
    ) {
        if !cursor.is_over(layout.bounds()) {
            if pressed(event) {
                shell.publish(Message::DismissTrackPlaylistPopup);
                shell.capture_event();
                return;
            }
            if matches!(event, Event::Mouse(mouse::Event::WheelScrolled { .. })) {
                shell.publish(Message::DismissTrackPlaylistPopup);
                return;
            }
        }
        self.content.as_widget_mut().update(
            self.tree,
            event,
            layout,
            cursor,
            renderer,
            clipboard,
            shell,
            &layout.bounds(),
        );
    }

    fn mouse_interaction(
        &self,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &Renderer,
    ) -> mouse::Interaction {
        self.content.as_widget().mouse_interaction(
            self.tree,
            layout,
            cursor,
            &layout.bounds(),
            renderer,
        )
    }
}

fn pressed(event: &Event) -> bool {
    matches!(
        event,
        Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left))
            | Event::Touch(iced::touch::Event::FingerPressed { .. })
    )
}

fn press_in(event: &Event, cursor: mouse::Cursor, bounds: Rectangle) -> bool {
    pressed(event) && cursor.is_over(bounds)
}

#[cfg(test)]
mod tests {
    use iced::touch::{Event::FingerPressed, Finger};

    use super::*;

    #[test]
    fn it_hits_the_playlist_anchor_using_the_scrolled_touch_cursor() {
        let bounds = Rectangle::new(Point::new(40.0, 640.0), Size::new(160.0, 28.0));
        let finger = Event::Touch(FingerPressed {
            id: Finger(0),
            position: Point::new(60.0, 240.0),
        });
        let cursor = mouse::Cursor::Available(Point::new(60.0, 644.0));

        assert!(press_in(&finger, cursor, bounds));
    }

    #[test]
    fn it_dismisses_the_playlist_popup_when_scrolling_outside_it() {
        let mut content = iced::widget::space().height(80).into();
        let mut tree = Tree::new(&content);
        let renderer = Renderer::new(iced::Font::DEFAULT, iced::Pixels(16.0));
        let mut popup = Popup {
            content: &mut content,
            tree: &mut tree,
            anchor: Rectangle::new(Point::new(40.0, 40.0), Size::new(160.0, 28.0)),
        };
        let node = popup.layout(&renderer, Size::new(400.0, 400.0));
        let cursor = mouse::Cursor::Available(Point::new(240.0, 80.0));
        let wheel = Event::Mouse(mouse::Event::WheelScrolled {
            delta: mouse::ScrollDelta::Lines { x: 0.0, y: -1.0 },
        });
        let mut messages = Vec::new();
        popup.update(
            &wheel,
            Layout::new(&node),
            cursor,
            &renderer,
            &mut iced::advanced::clipboard::Null,
            &mut Shell::new(&mut messages),
        );

        assert!(matches!(
            messages.as_slice(),
            [Message::DismissTrackPlaylistPopup]
        ));
    }
}
