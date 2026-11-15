use iced::{
    Alignment, Color, Element, Fill, Font, Pixels, Theme, border, font,
    widget::{Space, button, container, row, svg, text, tooltip},
};

use crate::{
    appearance::Appearance,
    catalogue::Library,
    desktop::{Message, library_sources::Operation, source_removal::Launcher},
};

use super::{
    ICON_TOOLTIP_TEXT_SIZE, Icon, action, control_opacity, ellipsized_label::EllipsizedLabel,
    icon_tinted, panel, pulse_row_fills, selection_fill, source_removal_button::RemovalButton,
};

const HEIGHT: f32 = 32.0;
const SELECTION_EDGE: f32 = 2.0;
const COLUMN_GAP: f32 = 7.0;
const GLYPH_SIZE: u32 = 15;
const NAME_SIZE: f32 = 12.0;
const NAME_LINE_HEIGHT: Pixels = Pixels(17.0);
const COUNT_SIZE: f32 = 9.0;
const COUNT_LINE_HEIGHT: Pixels = Pixels(13.0);

// The extra pixel in the top inset aligns Iced's text ink with the browser baseline.
const ROW_PADDING: iced::Padding = iced::Padding {
    top: 6.0,
    bottom: 4.0,
    left: 6.0,
    right: 6.0,
};

#[derive(Clone, Copy)]
pub(super) struct Colors {
    accent: Color,
    accent_ink: Color,
    selected_fill: Color,
    hover: Color,
    muted: Color,
    danger: Color,
}

impl Colors {
    pub(super) fn for_appearance(appearance: &Appearance) -> Self {
        const HOVER_OPACITY: f32 = 0.04;
        let palette = appearance.theme().palette();
        let hover = pulse_row_fills(appearance).map_or_else(
            || palette.text.scale_alpha(HOVER_OPACITY),
            |fills| fills.hover,
        );
        Self {
            accent: palette.primary,
            accent_ink: appearance.accent_ink(),
            selected_fill: selection_fill(appearance),
            hover,
            muted: appearance.surfaces().muted,
            danger: appearance.danger_ink(),
        }
    }
}

pub(super) fn view(
    source: &Library,
    count: Option<u64>,
    current: bool,
    enabled: bool,
    colors: Colors,
) -> Element<'_, Message> {
    let glyph = if source.kind == "dropbox" {
        Icon::Cloud
    } else {
        Icon::Folder
    };
    let count = count
        .map(|count| format!("{count} {}", if count == 1 { "track" } else { "tracks" }))
        .unwrap_or_default();
    let (edge, background, glyph_ink) = if current {
        (colors.accent, colors.selected_fill, colors.accent_ink)
    } else {
        (Color::TRANSPARENT, Color::TRANSPARENT, colors.muted)
    };
    let label = EllipsizedLabel::new(source.name.as_str(), NAME_SIZE)
        .font(Font {
            weight: font::Weight::Semibold,
            ..Font::with_name("Manrope")
        })
        .line_height(NAME_LINE_HEIGHT);
    let name_and_count = row![
        icon_tinted(glyph, move |_| glyph_ink)
            .width(GLYPH_SIZE)
            .height(GLYPH_SIZE),
        Element::new(label),
        text(count)
            .size(COUNT_SIZE)
            .font(crate::fonts::TABLE_FIGURES)
            .line_height(COUNT_LINE_HEIGHT)
            .color(colors.muted),
    ]
    .spacing(COLUMN_GAP)
    .height(Fill)
    .align_y(Alignment::Center);
    container(
        row![
            container(Space::new())
                .width(SELECTION_EDGE)
                .height(HEIGHT)
                .style(move |_| container::Style::default().background(edge)),
            button(name_and_count)
                .width(Fill)
                .height(HEIGHT)
                .padding(ROW_PADDING)
                .style(move |theme, status| source_button(theme, status, colors.hover))
                .on_press_maybe(
                    enabled.then(|| Message::SourceOperation(Operation::Select(source.id.clone())))
                ),
            remove_button(&source.id, enabled, colors),
        ]
        .align_y(Alignment::Center),
    )
    .width(Fill)
    .style(move |_| container::Style::default().background(background))
    .into()
}

fn source_button(theme: &Theme, status: button::Status, hover: Color) -> button::Style {
    let hovered = matches!(status, button::Status::Hovered | button::Status::Pressed);
    button::Style {
        background: hovered.then(|| hover.into()),
        border: border::rounded(0),
        ..action(theme, status)
    }
}

fn remove_button(source_id: &str, enabled: bool, colors: Colors) -> Element<'static, Message> {
    const SIZE: u32 = 22;
    const HOVER_OPACITY: f32 = 0.08;
    // The SVG covers the button so its ink responds throughout the hit area.
    const CLOSE: &[u8] = br#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="-10 -10 44 44" fill="none" stroke="black" stroke-width="2" stroke-linecap="round"><path d="m18 6-12 12M6 6l12 12"/></svg>"#;
    let Colors { muted, danger, .. } = colors;
    let icon = svg(svg::Handle::from_memory(CLOSE))
        .width(SIZE)
        .height(SIZE)
        .opacity(control_opacity(enabled))
        .style(move |_, status| svg::Style {
            color: Some(if enabled && status == svg::Status::Hovered {
                danger
            } else {
                muted
            }),
        });
    tooltip(
        Element::new(RemovalButton::new(
            button(icon)
                .padding(0)
                .style(move |_, status| button::Style {
                    background: matches!(status, button::Status::Hovered | button::Status::Pressed)
                        .then(|| danger.scale_alpha(HOVER_OPACITY).into()),
                    ..button::Style::default()
                }),
            Launcher::Sidebar(source_id.to_owned()),
            enabled,
        )),
        text("Remove source").size(ICON_TOOLTIP_TEXT_SIZE),
        tooltip::Position::FollowCursor,
    )
    .style(panel)
    .into()
}
