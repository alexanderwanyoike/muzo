use iced::{
    Event, Length, Rectangle, Renderer, Size, Theme,
    advanced::{
        Clipboard, Layout, Renderer as _, Shell, Widget, layout, mouse, renderer,
        widget::{Id, Operation, Tree, operation::Focusable, tree},
    },
    border,
    keyboard::{self, Key, key::Named},
    touch,
    widget::Button,
    window,
};

use crate::desktop::{Message, source_removal::Launcher};

use super::SOURCE_ACTION_FOCUS_TINT;

pub(super) struct RemovalButton<'a> {
    button: Button<'a, Message>,
    launcher: Launcher,
    id: Id,
    enabled: bool,
}

struct State {
    id: Id,
    focused: bool,
    reveal: bool,
    space_armed: bool,
}

impl Focusable for State {
    fn is_focused(&self) -> bool {
        self.focused
    }

    fn focus(&mut self) {
        self.reveal |= !std::mem::replace(&mut self.focused, true);
    }

    fn unfocus(&mut self) {
        self.focused = false;
        self.reveal = false;
        self.space_armed = false;
    }
}

impl<'a> RemovalButton<'a> {
    pub(super) fn new(button: Button<'a, Message>, launcher: Launcher, enabled: bool) -> Self {
        let button = button.on_press_maybe(
            enabled.then(|| Message::RequestSourceRemoval(launcher.source_id().to_owned())),
        );
        Self {
            id: launcher.widget_id(),
            button,
            launcher,
            enabled,
        }
    }
}

impl Widget<Message, Theme, Renderer> for RemovalButton<'_> {
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<State>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(State {
            id: self.id.clone(),
            focused: false,
            reveal: false,
            space_armed: false,
        })
    }

    fn children(&self) -> Vec<Tree> {
        vec![Tree::new(
            &self.button as &dyn Widget<Message, Theme, Renderer>,
        )]
    }

    fn diff(&self, tree: &mut Tree) {
        let state = tree.state.downcast_mut::<State>();
        // The row position can outlive its source when the list is filtered.
        if self.id != state.id || !self.enabled {
            state.unfocus();
            state.id = self.id.clone();
        }
        tree.children[0].diff(&self.button as &dyn Widget<Message, Theme, Renderer>);
    }

    fn size(&self) -> Size<Length> {
        self.button.size()
    }

    fn layout(
        &mut self,
        tree: &mut Tree,
        renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        self.button.layout(&mut tree.children[0], renderer, limits)
    }

    fn operate(
        &mut self,
        tree: &mut Tree,
        layout: Layout<'_>,
        renderer: &Renderer,
        operation: &mut dyn Operation,
    ) {
        if self.enabled {
            operation.focusable(
                Some(&self.id),
                layout.bounds(),
                tree.state.downcast_mut::<State>(),
            );
        }
        self.button
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
        if state.focused {
            match event {
                Event::Mouse(mouse::Event::ButtonPressed(_))
                | Event::Touch(touch::Event::FingerPressed { .. }) => {
                    state.unfocus();
                    shell.request_redraw();
                }
                Event::Window(window::Event::Unfocused) => state.space_armed = false,
                Event::Window(window::Event::RedrawRequested(_))
                    if std::mem::take(&mut state.reveal) =>
                {
                    shell.publish(Message::RevealSourceRemovalLauncher(self.launcher.clone()));
                }
                _ => {}
            }
            if self.enabled
                && let Some(activate) = state.activation(event)
            {
                if activate {
                    shell.publish(Message::RequestSourceRemovalFromKeyboard(
                        self.launcher.clone(),
                    ));
                }
                shell.capture_event();
                return;
            }
        }
        self.button.update(
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
        self.button.draw(
            &tree.children[0],
            renderer,
            theme,
            style,
            layout,
            cursor,
            viewport,
        );
        if self.enabled && tree.state.downcast_ref::<State>().focused {
            const FILL_INSET: f32 = 2.0;
            renderer.fill_quad(
                renderer::Quad {
                    bounds: layout.bounds().shrink(FILL_INSET),
                    border: border::rounded(3),
                    ..renderer::Quad::default()
                },
                theme.palette().text.scale_alpha(SOURCE_ACTION_FOCUS_TINT),
            );
        }
    }

    fn mouse_interaction(
        &self,
        tree: &Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
        renderer: &Renderer,
    ) -> mouse::Interaction {
        self.button
            .mouse_interaction(&tree.children[0], layout, cursor, viewport, renderer)
    }
}

impl State {
    fn activation(&mut self, event: &Event) -> Option<bool> {
        match event {
            Event::Keyboard(keyboard::Event::KeyPressed {
                key,
                repeat,
                modifiers,
                ..
            }) if !(modifiers.control() || modifiers.logo() || modifiers.alt()) => match key {
                Key::Named(Named::Enter) => Some(!repeat),
                Key::Named(Named::Space) => {
                    self.space_armed |= !repeat;
                    Some(false)
                }
                Key::Named(Named::Tab) => {
                    self.space_armed = false;
                    None
                }
                _ => None,
            },
            Event::Keyboard(keyboard::Event::KeyReleased {
                key: Key::Named(Named::Space),
                modifiers,
                ..
            }) => {
                let armed = std::mem::take(&mut self.space_armed);
                armed.then_some(!(modifiers.control() || modifiers.logo() || modifiers.alt()))
            }
            _ => None,
        }
    }
}
