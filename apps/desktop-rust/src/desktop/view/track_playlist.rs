use iced::{
    Alignment, Element, Fill, border,
    widget::{column, container, pick_list, row, text},
};

use crate::{
    appearance::Surfaces,
    catalogue::Playlist,
    desktop::{Message, library_sources::PlaylistPicker},
};

use super::library::settings_button;

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
    let cancel = settings_button("Cancel", surfaces)
        .on_press_maybe((!pending).then_some(Message::ClosePlaylistPicker));
    let body = if playlists.is_empty() {
        row![text("No playlists yet.").size(13), cancel]
    } else {
        let select = pick_list::<Playlist, _, _, _, _, _>(
            if pending { &[] } else { playlists },
            selected.cloned(),
            |playlist| Message::ChooseTrackPlaylist(playlist.id),
        )
        .text_size(11)
        .padding([7, 9])
        .style(move |theme, status| {
            let mut style = pick_list::default(theme, status);
            style.background = surfaces.raised.into();
            style.border = border::rounded(4).width(1).color(surfaces.line);
            style
        });
        let submit = settings_button(
            if pending {
                "Adding..."
            } else {
                "Add to playlist"
            },
            surfaces,
        )
        .on_press_maybe(
            (!busy && !pending && selected.is_some()).then_some(Message::SubmitPlaylistPicker),
        );
        row![
            column![text("Playlist").size(11).color(surfaces.muted), select].spacing(5),
            submit,
            cancel,
        ]
    };
    let mut content = column![body.spacing(8).align_y(Alignment::End)].spacing(8);
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
