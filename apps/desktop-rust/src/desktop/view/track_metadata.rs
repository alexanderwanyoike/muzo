use iced::{
    Fill,
    widget::{Space, column, container, row, text},
};

use crate::{
    appearance::Surfaces,
    desktop::{
        Message,
        library_sources::TrackEditor,
        track_editor_focus::{Control, Field},
    },
};

use super::{
    focus_ring,
    library::{bordered_button, settings_button, settings_input},
};

pub(super) fn editor(
    editor: &TrackEditor,
    busy: bool,
    saving: bool,
    surfaces: Surfaces,
) -> iced::widget::Container<'_, Message> {
    let save = (!busy && !saving).then_some(Message::SaveTrackEdit(false));
    let field = |field: Field| {
        column![
            text(field.label()).size(11).color(surfaces.muted),
            settings_input(&editor.fields[field.index()], surfaces)
                .id(field.id())
                .size(11)
                .padding([8, 9])
                .on_input_maybe((!saving).then_some(move |value| Message::EditField(field, value)))
                .on_submit_maybe(save.clone()),
        ]
        .width(Fill)
        .spacing(6)
    };
    let control = |kind: Control, label, enabled| {
        let focused = enabled && editor.action_focus == Some(kind);
        container(
            settings_button(label, surfaces)
                .style(move |theme, status| {
                    focus_ring(bordered_button(theme, status, surfaces), theme, focused)
                })
                .on_press_maybe(kind.message().filter(|_| enabled)),
        )
        .id(kind.id())
    };
    let mut actions = row![control(Control::Save, "Save metadata", save.is_some())].spacing(8);
    if editor.overridden {
        actions = actions.push(control(Control::Clear, "Clear override", save.is_some()));
    }
    let actions = actions.push(control(Control::Cancel, "Cancel", !saving));
    let mut body = column![
        row![
            field(Field::Title),
            field(Field::Artist),
            field(Field::Album)
        ]
        .spacing(12),
        row![field(Field::Track), field(Field::Disc), field(Field::Genre)].spacing(12),
        row![
            field(Field::Year),
            Space::new().width(Fill),
            Space::new().width(Fill)
        ]
        .spacing(12),
    ]
    .spacing(12);
    if let Some(error) = &editor.error {
        body = body.push(text(error).size(12));
    }
    container(body.push(actions)).padding(iced::Padding {
        top: 17.0,
        left: 35.0,
        bottom: 18.0,
        ..Default::default()
    })
}
