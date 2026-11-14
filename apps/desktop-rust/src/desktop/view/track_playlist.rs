use iced::{
    Alignment, Element, Fill, border,
    widget::{column, container, pick_list, row, text},
};

use crate::{
    appearance::Surfaces,
    catalogue::Playlist,
    desktop::{Message, library_sources::PlaylistPicker, playlist_picker_focus::Control},
};

use super::{
    focus_ring,
    library::{bordered_button, settings_button},
};

pub(super) fn picker<'a>(
    draft: &'a PlaylistPicker,
    playlists: &'a [Playlist],
    busy: bool,
    pending: bool,
    surfaces: Surfaces,
) -> Element<'a, Message> {
    let selected = draft
        .playlist
        .as_deref()
        .and_then(|id| playlists.iter().find(|playlist| playlist.id == id));
    let button = |control: Control, label, enabled| {
        let focused = enabled && draft.focus == Some(control);
        container(
            settings_button(label, surfaces)
                .style(move |theme, status| {
                    focus_ring(bordered_button(theme, status, surfaces), theme, focused)
                })
                .on_press_maybe(control.message().filter(|_| enabled)),
        )
        .id(control.id())
    };
    let cancel = button(Control::Cancel, "Cancel", !pending);
    let body = if playlists.is_empty() {
        row![text("No playlists yet.").size(13), cancel]
    } else {
        let focused = !pending && draft.focus == Some(Control::Choice);
        let select =
            pick_list::<Playlist, _, _, _, _, _>(playlists, selected.cloned(), |playlist| {
                Message::ChooseTrackPlaylist(playlist.id)
            })
            .text_size(11)
            .padding([7, 9])
            .style(move |theme, status| {
                let mut style = pick_list::default(theme, status);
                style.background = surfaces.raised.into();
                style.border = if focused {
                    border::rounded(4).width(2).color(style.text_color)
                } else {
                    border::rounded(4).width(1).color(surfaces.line)
                };
                style
            });
        let submit = button(
            Control::Add,
            if pending {
                "Adding..."
            } else {
                "Add to playlist"
            },
            !busy && !pending && selected.is_some(),
        );
        let choice =
            container(column![text("Playlist").size(11).color(surfaces.muted), select,].spacing(5))
                .id(Control::Choice.id());
        row![choice, row![submit, cancel].spacing(8)]
    };
    let mut content = column![body.spacing(8).align_y(Alignment::End).wrap()].spacing(8);
    if let Some(error) = &draft.error {
        content = content.push(text(error).size(12));
    }
    container(content)
        .width(Fill)
        .padding(iced::Padding {
            top: 13.0,
            left: 35.0,
            bottom: 16.0,
            ..Default::default()
        })
        .into()
}
