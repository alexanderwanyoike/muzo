use iced::{
    Element,
    widget::{column, pick_list, row, text},
};

use crate::appearance::{ACCENTS, Appearance, MATERIALS, MODES};

use super::Message;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct OptionLabel(&'static str, &'static str);

impl std::fmt::Display for OptionLabel {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.1.fmt(formatter)
    }
}

pub fn controls(appearance: &Appearance) -> Element<'_, Message> {
    column![
        text("Appearance").size(19),
        row![
            choice("Material", &appearance.theme, MATERIALS, |value| {
                Message::Material(value.into())
            }),
            choice("Mode", &appearance.mode, MODES, |value| {
                Message::AppearanceMode(value.into())
            }),
            choice("Accent", &appearance.accent, ACCENTS, |value| {
                Message::Accent(value.into())
            }),
        ]
        .spacing(14),
    ]
    .spacing(10)
    .into()
}

fn choice<const N: usize>(
    label: &'static str,
    current: &str,
    options: [(&'static str, &'static str); N],
    select: impl Fn(&'static str) -> Message + 'static,
) -> Element<'static, Message> {
    let options = options.map(|(id, label)| OptionLabel(id, label));
    let selected = options.iter().find(|option| option.0 == current).copied();
    column![
        text(label).size(13),
        pick_list(options, selected, move |option| select(option.0)).text_size(13),
    ]
    .spacing(4)
    .into()
}
