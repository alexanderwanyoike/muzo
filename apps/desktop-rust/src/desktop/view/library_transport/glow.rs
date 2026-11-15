use iced::{
    Color, Element, Shadow, border, color,
    widget::{button, slider, stack},
};

use crate::appearance::Appearance;

use super::{Message, PLAY_BUTTON_EDGE, seek_halo::SeekHalo, volume_halo::VolumeHalo};

const PLAYBACK_BLUR: f32 = 10.0;
const PLAYBACK_OPACITY: f32 = 0.20;

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
            shadow: self.shadow(PLAYBACK_BLUR, PLAYBACK_OPACITY),
            ..button::Style::default()
        }
    }

    pub(super) fn surround_seek(
        self,
        control: Element<'static, Message>,
    ) -> Element<'static, Message> {
        stack![control]
            .push_under(Element::new(SeekHalo(self.accent)))
            .into()
    }

    pub(super) fn volume_ink(self, mut style: slider::Style) -> slider::Style {
        style.rail.backgrounds.0 = self.outline.into();
        style.handle.background = self.outline.into();
        style
    }

    pub(super) fn surround_volume(
        self,
        control: Element<'static, Message>,
        volume: f32,
    ) -> Element<'static, Message> {
        stack![control]
            .push_under(Element::new(VolumeHalo::new(self.accent, volume)))
            .into()
    }

    fn shadow(self, blur_radius: f32, opacity: f32) -> Shadow {
        Shadow {
            color: self.accent.scale_alpha(opacity),
            blur_radius,
            ..Shadow::default()
        }
    }
}
