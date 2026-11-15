use iced::{
    Color, Element, Shadow, border, color,
    widget::{Space, button, container, stack},
};

use crate::appearance::Appearance;

use super::{Message, PLAY_BUTTON_EDGE};

#[derive(Clone, Copy)]
pub(super) struct Glow {
    accent: Color,
    outline: Color,
    highlight: Color,
}

impl Glow {
    pub(super) const FOREGROUND: Color = color!(0x10151b);

    pub(super) fn for_appearance(appearance: &Appearance) -> Option<Self> {
        Some(Self {
            highlight: appearance.glow_highlight()?,
            accent: appearance.theme().palette().primary,
            outline: appearance.accent_ink(),
        })
    }

    pub(super) fn playback_style(self, status: button::Status) -> button::Style {
        let fill = match status {
            button::Status::Hovered | button::Status::Pressed => self.highlight,
            _ => self.accent,
        };
        button::Style {
            background: Some(fill.into()),
            text_color: Self::FOREGROUND,
            border: border::rounded(PLAY_BUTTON_EDGE / 2.0)
                .width(1.0)
                .color(self.outline),
            shadow: self.shadow(14.0, 0.32),
            ..button::Style::default()
        }
    }

    pub(super) fn surround_playback(
        self,
        control: Element<'static, Message>,
    ) -> Element<'static, Message> {
        let halo = container(Space::new())
            .center(PLAY_BUTTON_EDGE)
            .style(move |_| {
                container::Style::default()
                    .border(border::rounded(PLAY_BUTTON_EDGE / 2.0))
                    .shadow(self.shadow(28.0, 0.12))
            });
        stack![control].push_under(halo).into()
    }

    fn shadow(self, blur_radius: f32, opacity: f32) -> Shadow {
        Shadow {
            color: self.accent.scale_alpha(opacity),
            blur_radius,
            ..Shadow::default()
        }
    }
}
