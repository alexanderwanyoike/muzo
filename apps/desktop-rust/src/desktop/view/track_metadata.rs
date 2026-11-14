use iced::{
    Fill,
    widget::{Space, column, container, row, text},
};

use crate::{
    appearance::Surfaces,
    desktop::{Message, library_sources::TrackEditor},
};

use super::{
    error_line,
    library::{settings_button, settings_input},
};

pub(super) fn editor(
    editor: &TrackEditor,
    busy: bool,
    saving: bool,
    surfaces: Surfaces,
) -> iced::widget::Container<'_, Message> {
    let save = (!busy && !saving).then_some(Message::SaveTrackEdit(false));
    let field = |index: usize, label| {
        column![
            text(label).size(11).color(surfaces.muted),
            settings_input(&editor.fields[index], surfaces)
                .size(11)
                .padding([8, 9])
                .on_input_maybe((!saving).then_some(move |value| Message::EditField(index, value)))
                .on_submit_maybe(save.clone()),
        ]
        .width(Fill)
        .spacing(6)
    };
    let mut actions =
        row![settings_button("Save metadata", surfaces).on_press_maybe(save.clone())].spacing(8);
    if editor.overridden {
        actions = actions.push(
            settings_button("Clear override", surfaces)
                .on_press_maybe((!busy && !saving).then_some(Message::SaveTrackEdit(true))),
        );
    }
    let actions = actions.push(
        settings_button("Cancel", surfaces)
            .on_press_maybe((!saving).then_some(Message::CloseTrackEdit)),
    );
    let mut body = column![
        row![field(0, "Title"), field(1, "Artist"), field(2, "Album")].spacing(12),
        row![field(3, "Track"), field(4, "Disc"), field(5, "Genre")].spacing(12),
        row![
            field(6, "Year"),
            Space::new().width(Fill),
            Space::new().width(Fill)
        ]
        .spacing(12),
    ]
    .spacing(12);
    if let Some(error) = &editor.error {
        body = body.push(error_line(Some(error)));
    }
    container(body.push(actions)).padding(iced::Padding {
        top: 17.0,
        left: 35.0,
        bottom: 18.0,
        ..Default::default()
    })
}
