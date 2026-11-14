use iced::{
    Fill,
    widget::{Space, column, container, row, text},
};

use crate::{appearance::Surfaces, desktop::Message};

use super::library::{settings_button, settings_input};

pub(super) fn editor(
    fields: &[String; 7],
    busy: bool,
    surfaces: Surfaces,
) -> iced::widget::Container<'_, Message> {
    let save = (!busy).then_some(Message::SaveTrackEdit(false));
    let field = |index: usize, label| {
        column![
            text(label).size(11).color(surfaces.muted),
            settings_input(fields[index].as_str(), surfaces)
                .on_input(move |value| Message::EditField(index, value))
                .on_submit_maybe(save.clone()),
        ]
        .width(Fill)
        .spacing(5)
    };
    container(
        column![
            row![field(0, "Title"), field(1, "Artist"), field(2, "Album")].spacing(12),
            row![field(3, "Track"), field(4, "Disc"), field(5, "Genre")].spacing(12),
            row![
                field(6, "Year"),
                Space::new().width(Fill),
                Space::new().width(Fill)
            ]
            .spacing(12),
            row![
                settings_button("Save metadata", surfaces).on_press_maybe(save),
                settings_button("Clear override", surfaces)
                    .on_press_maybe((!busy).then_some(Message::SaveTrackEdit(true))),
                settings_button("Cancel", surfaces).on_press(Message::CloseTrackEdit),
            ]
            .spacing(8),
        ]
        .spacing(12),
    )
    .padding(iced::Padding {
        top: 15.0,
        left: 35.0,
        bottom: 15.0,
        ..Default::default()
    })
}
