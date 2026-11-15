use iced::{
    Color, Element, Fill, Shadow, border, color,
    widget::{Space, button, container, row, slider, stack},
};

use crate::appearance::Appearance;

use super::{
    Message, PLAY_BUTTON_EDGE, VOLUME_INPUT_HEIGHT,
    compact::{SLIDER_RAIL_HEIGHT, VOLUME_THUMB_RADIUS},
    seek_halo::SeekHalo,
    volume::thumb_travel,
};

const PLAYBACK_BLUR: f32 = 10.0;
const VOLUME_BLUR: f32 = 5.0;
const PLAYBACK_OPACITY: f32 = 0.20;
const VOLUME_OPACITY: f32 = 0.32;

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
        width: f32,
        volume: f32,
    ) -> Element<'static, Message> {
        let shadow = self.shadow(VOLUME_BLUR, VOLUME_OPACITY);
        let diameter = VOLUME_THUMB_RADIUS * 2.0;
        let thumb = container(Space::new()).center(diameter).style(move |_| {
            container::Style::default()
                .border(border::rounded(VOLUME_THUMB_RADIUS))
                .shadow(shadow)
        });
        let thumb_shadow = container(row![
            Space::new().width(thumb_travel(width) * volume),
            thumb
        ])
        .center_y(VOLUME_INPUT_HEIGHT);
        stack![control]
            .push_under(thumb_shadow)
            .push_under(rail_shadow(VOLUME_INPUT_HEIGHT, shadow))
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

fn rail_shadow(input_height: f32, shadow: Shadow) -> container::Container<'static, Message> {
    container(
        container(Space::new())
            .width(Fill)
            .height(SLIDER_RAIL_HEIGHT)
            .style(move |_| {
                container::Style::default()
                    .border(border::rounded(SLIDER_RAIL_HEIGHT / 2.0))
                    .shadow(shadow)
            }),
    )
    .center_y(input_height)
}
