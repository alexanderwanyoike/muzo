use iced::{
    Color, ContentFit, Element, Event, Fill, Font, Length, Padding, Pixels, border, color,
    keyboard::{self, Key, Modifiers, key::Named},
    widget::{button, column, container, row, scrollable, stack, svg, text},
    window,
};

use crate::{
    appearance::{Appearance, Surfaces},
    desktop::{
        Message,
        library_sources::Operation,
        source_removal::{DETAILS, Removal, Scroll},
        windows::LIBRARY_MIN,
    },
};

use super::{bold, label_text::replace_embedded_breaks, library, modal::Modal, pulse_row_fills};

const DESCRIPTION: &str = "This removes the source, its indexed tracks, metadata edits and play counts from Muzo. Your music files stay where they are. Playlist entries remain, marked unavailable.";
const PANEL_WIDTH: f32 = 440.0;
const VIEW_MARGIN: f32 = 30.0;
const CONTENT_INSET: f32 = 25.0;
const TEXT_GAP: f32 = 12.0;
const ACTION_INSET: f32 = 24.0;
const ACTION_GAP: f32 = 8.0;
const ACTION_HEIGHT: f32 = 30.0;
const FOCUS_OUTSET: f32 = 5.0;
const SHADOW_INSET: f32 = 88.0;
const SHADOW: &[u8] = include_bytes!("../../../assets/materials/dialog-shadow.svg");
const MAX_COPY_HEIGHT: f32 =
    LIBRARY_MIN.height - 2.0 * (SHADOW_INSET + CONTENT_INSET) - ACTION_INSET - ACTION_HEIGHT;
const LINE_SCROLL: f32 = 23.0;
const PAGE_SCROLL: f32 = MAX_COPY_HEIGHT - LINE_SCROLL;
const SCROLLBAR_GAP: f32 = 4.0;

pub(in crate::desktop) fn layer<'a>(
    page: Element<'a, Message>,
    removal: Option<&'a Removal>,
    appearance: &Appearance,
) -> Element<'a, Message> {
    Modal::wrap(
        page,
        removal.map(|removal| (view(removal, appearance), key_action as _)),
    )
}

fn view<'a>(removal: &'a Removal, appearance: &Appearance) -> Element<'a, Message> {
    let surfaces = appearance.surfaces();
    let (outline, hover) = pulse_row_fills(appearance).map_or_else(
        || (appearance.accent_ink(), None),
        |fills| {
            let neutral = if appearance.mode == "dark" {
                Color::WHITE
            } else {
                color!(0x171717)
            };
            (neutral, Some(fills.hover))
        },
    );
    let danger = if appearance.mode == "dark" {
        color!(0xf1a49a)
    } else {
        color!(0xab3e36)
    };
    let title = format!(
        "Remove {}?",
        replace_embedded_breaks(removal.source.name.as_str())
    );
    let location = replace_embedded_breaks(removal.source.location.to_string_lossy());
    let mut copy = column![
        text(title)
            .size(17)
            .line_height(Pixels(23.0))
            .font(bold())
            .wrapping(text::Wrapping::WordOrGlyph),
        text(DESCRIPTION)
            .size(12)
            .line_height(Pixels(21.6))
            .color(surfaces.muted)
            .wrapping(text::Wrapping::WordOrGlyph),
        text(location)
            .size(11)
            .line_height(Pixels(13.0))
            .font(Font::MONOSPACE)
            .color(surfaces.muted)
            .wrapping(text::Wrapping::WordOrGlyph),
    ]
    .spacing(TEXT_GAP)
    .width(Fill);
    if let Some(error) = removal.error.as_deref() {
        copy = copy.push(
            text(replace_embedded_breaks(error))
                .size(12)
                .line_height(Pixels(21.6))
                .color(danger)
                .wrapping(text::Wrapping::WordOrGlyph),
        );
    }

    let cancel_colors = ActionColors {
        surfaces,
        outline,
        hover,
        danger: None,
    };
    let cancel = action(
        "Cancel",
        (!removal.busy).then_some(Message::CancelSourceRemoval),
        removal.show_keyboard_focus && removal.cancel_focused,
        cancel_colors,
    );
    let confirm = action(
        if removal.busy {
            "Removing..."
        } else {
            "Remove source"
        },
        (!removal.busy)
            .then(|| Message::SourceOperation(Operation::Remove(removal.source.id.clone()))),
        removal.show_keyboard_focus && !removal.cancel_focused,
        ActionColors {
            danger: Some(danger),
            ..cancel_colors
        },
    );
    let actions = row![cancel, confirm].spacing(ACTION_GAP - 2.0 * FOCUS_OUTSET);
    let inner_inset = CONTENT_INSET - FOCUS_OUTSET;
    let copy = container(
        scrollable(copy)
            .id(DETAILS)
            .width(Fill)
            .spacing(SCROLLBAR_GAP)
            .height(Length::Shrink),
    )
    .padding([0.0, FOCUS_OUTSET])
    .max_height(MAX_COPY_HEIGHT);
    let panel = container(
        column![copy, container(actions).align_right(Fill)].spacing(ACTION_INSET - FOCUS_OUTSET),
    )
    .padding(Padding {
        top: CONTENT_INSET,
        right: inner_inset,
        bottom: inner_inset,
        left: inner_inset,
    })
    .width(Fill)
    .max_width(PANEL_WIDTH)
    .style(move |_| library::framed(surfaces.raised, surfaces, 8));

    // Tiny-Skia paints a quad's built-in shadow outside its partial redraw clip.
    let shadow = svg(svg::Handle::from_memory(SHADOW))
        .width(Fill)
        .height(Fill)
        .content_fit(ContentFit::Fill);
    let shadowed_panel =
        stack![container(panel).padding(SHADOW_INSET).width(Fill)].push_under(shadow);
    let panel = container(shadowed_panel)
        .width(Fill)
        .max_width(PANEL_WIDTH + 2.0 * SHADOW_INSET);

    container(panel)
        .center(Fill)
        .padding([0.0, VIEW_MARGIN])
        .style(|_| container::Style::default().background(Color::BLACK.scale_alpha(0.4)))
        .into()
}

#[derive(Clone, Copy)]
struct ActionColors {
    surfaces: Surfaces,
    outline: Color,
    hover: Option<Color>,
    danger: Option<Color>,
}

fn action(
    label: &'static str,
    message: Option<Message>,
    focused: bool,
    colors: ActionColors,
) -> Element<'static, Message> {
    let ActionColors {
        surfaces,
        outline,
        hover,
        danger,
    } = colors;
    let focused = focused && message.is_some();
    let control = button(text(label).size(13).line_height(Pixels(18.0)))
        .height(ACTION_HEIGHT)
        .padding([6, 12])
        .style(move |theme, status| {
            let mut style = library::bordered_button(theme, status, surfaces);
            if let Some(danger) = danger {
                style.background = Some(danger.into());
                style.border.color = danger;
                style.text_color = surfaces.raised;
            } else if status == button::Status::Hovered
                || hover.is_some() && status == button::Status::Pressed
            {
                style.background = Some(
                    hover
                        .unwrap_or_else(|| {
                            super::compact::tint(surfaces.raised, theme.palette().text, 0.05)
                        })
                        .into(),
                );
            }
            if status == button::Status::Disabled {
                style.background = style.background.map(|color| color.scale_alpha(0.4));
                style.border.color = style.border.color.scale_alpha(0.4);
                style.text_color = style.text_color.scale_alpha(0.4);
            }
            style
        })
        .on_press_maybe(message);
    container(control)
        .padding(FOCUS_OUTSET)
        .style(move |_| {
            let border = if focused {
                border::rounded(10).width(2).color(outline)
            } else {
                border::Border::default()
            };
            container::Style::default().border(border)
        })
        .into()
}

fn key_action(event: &Event) -> Option<Message> {
    let shortcut =
        |modifiers: Modifiers| modifiers.control() || modifiers.logo() || modifiers.alt();
    match event {
        Event::Keyboard(keyboard::Event::KeyPressed {
            key,
            repeat,
            modifiers,
            ..
        }) if !shortcut(*modifiers) => match key {
            Key::Named(Named::Escape) => Some(Message::CancelSourceRemoval),
            Key::Named(Named::Tab) => Some(Message::CycleSourceRemovalFocus),
            Key::Named(Named::Enter) if !repeat => Some(Message::ActivateSourceRemovalFocus),
            Key::Named(Named::Space) if !repeat => Some(Message::SetSourceRemovalSpaceArmed(true)),
            Key::Named(Named::ArrowUp) => {
                Some(Message::ScrollSourceRemoval(Scroll::By(-LINE_SCROLL)))
            }
            Key::Named(Named::ArrowDown) => {
                Some(Message::ScrollSourceRemoval(Scroll::By(LINE_SCROLL)))
            }
            Key::Named(Named::PageUp) => {
                Some(Message::ScrollSourceRemoval(Scroll::By(-PAGE_SCROLL)))
            }
            Key::Named(Named::PageDown) => {
                Some(Message::ScrollSourceRemoval(Scroll::By(PAGE_SCROLL)))
            }
            Key::Named(Named::Home) => Some(Message::ScrollSourceRemoval(Scroll::Start)),
            Key::Named(Named::End) => Some(Message::ScrollSourceRemoval(Scroll::End)),
            _ => None,
        },
        Event::Keyboard(keyboard::Event::KeyReleased {
            key: Key::Named(Named::Space),
            modifiers,
            ..
        }) if !shortcut(*modifiers) => Some(Message::ReleaseSourceRemovalSpace),
        Event::Keyboard(keyboard::Event::KeyReleased {
            key: Key::Named(Named::Space),
            ..
        })
        | Event::Window(window::Event::Unfocused) => {
            Some(Message::SetSourceRemovalSpaceArmed(false))
        }
        _ => None,
    }
}
