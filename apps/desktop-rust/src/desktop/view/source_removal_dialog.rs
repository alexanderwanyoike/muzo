use iced::{
    Color, ContentFit, Element, Event, Fill, Font, Padding, Pixels, border, color,
    keyboard::{self, Key, Modifiers, key::Named},
    widget::{button, column, container, row, stack, svg, text},
    window,
};

use crate::{
    appearance::{Appearance, Surfaces},
    desktop::{Message, library_sources::Operation, source_removal::Removal},
};

use super::{bold, label_text::replace_embedded_breaks, library, modal::Modal};

const DESCRIPTION: &str = "This removes the source, its indexed tracks, metadata edits and play counts from Muzo. Your music files stay where they are. Playlist entries remain, marked unavailable.";
const PANEL_WIDTH: f32 = 440.0;
const VIEW_MARGIN: f32 = 30.0;
const CONTENT_INSET: f32 = 25.0;
const TEXT_GAP: f32 = 12.0;
const ACTION_INSET: f32 = 24.0;
const ACTION_GAP: f32 = 8.0;
const FOCUS_OUTSET: f32 = 5.0;
const SHADOW_INSET: f32 = 88.0;
const SHADOW: &[u8] = include_bytes!("../../../assets/materials/dialog-shadow.svg");

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
    let accent = appearance.accent_ink();
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

    let cancel = action(
        "Cancel",
        (!removal.busy).then_some(Message::CancelSourceRemoval),
        removal.show_keyboard_focus && removal.cancel_focused,
        (surfaces, accent, None),
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
        (surfaces, accent, Some(danger)),
    );
    let actions = row![cancel, confirm].spacing(ACTION_GAP - 2.0 * FOCUS_OUTSET);
    let inner_inset = CONTENT_INSET - FOCUS_OUTSET;
    let panel = container(
        column![
            container(copy).padding([0.0, FOCUS_OUTSET]),
            container(actions).align_right(Fill),
        ]
        .spacing(ACTION_INSET - FOCUS_OUTSET),
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

fn action(
    label: &'static str,
    message: Option<Message>,
    focused: bool,
    colors: (Surfaces, Color, Option<Color>),
) -> Element<'static, Message> {
    let (surfaces, accent, danger) = colors;
    let focused = focused && message.is_some();
    let control = button(text(label).size(13).line_height(Pixels(18.0)))
        .height(30)
        .padding([6, 12])
        .style(move |theme, status| {
            let mut style = library::bordered_button(theme, status, surfaces);
            if let Some(danger) = danger {
                style.background = Some(danger.into());
                style.border.color = danger;
                style.text_color = surfaces.raised;
            } else if status == button::Status::Hovered {
                style.background =
                    Some(super::compact::tint(surfaces.raised, theme.palette().text, 0.05).into());
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
                border::rounded(10).width(2).color(accent)
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
