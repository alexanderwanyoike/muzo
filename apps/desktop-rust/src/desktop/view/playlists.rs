use iced::{
    Alignment, Element, Fill,
    widget::{button, column, container, row, scrollable, text, text_input},
};

use crate::desktop::{
    Message,
    library_sources::{Operation, Snapshot, Sources},
};

use super::{action, bold, muted, panel, primary, selected};

pub fn sidebar(sources: &Sources) -> Element<'_, Message> {
    let entries = sources
        .snapshot
        .iter()
        .flat_map(|snapshot| &snapshot.playlists)
        .map(|playlist| {
            let active = sources
                .snapshot
                .as_ref()
                .and_then(|snapshot| snapshot.selected_playlist.as_deref())
                == Some(&playlist.id);
            button(text(&playlist.name).size(13))
                .padding([6, 9])
                .width(Fill)
                .style(move |theme, status| {
                    if active {
                        selected(theme, status)
                    } else {
                        action(theme, status)
                    }
                })
                .on_press_maybe((!sources.busy()).then(|| {
                    Message::SourceOperation(Operation::SelectPlaylist(playlist.id.clone()))
                }))
                .into()
        });
    column![
        text("Playlists").font(bold()).size(15),
        scrollable(column(entries).spacing(2)).height(100),
        text_input("Playlist name", &sources.playlist_name)
            .size(12)
            .on_input(Message::PlaylistName),
        button(text("Create playlist").size(12))
            .style(action)
            .on_press_maybe(
                (!sources.busy() && !sources.playlist_name.trim().is_empty())
                    .then_some(Message::CreatePlaylist)
            ),
    ]
    .spacing(6)
    .into()
}

pub fn tracks(snapshot: &Snapshot, busy: bool) -> iced::widget::Column<'_, Message> {
    let Some(playlist) = &snapshot.selected_playlist else {
        return column![];
    };
    let mut playable = 0;
    let rows = snapshot
        .playlist_entries
        .iter()
        .enumerate()
        .map(|(index, entry)| {
            let (label, play) = if let Some(track) = &entry.track {
                let play = Message::PlayPlaylist(playable);
                playable += 1;
                (track.title.as_str(), Some(play))
            } else {
                ("Unavailable track", None)
            };
            row![
                button(text(label).size(14).width(Fill))
                    .width(Fill)
                    .style(action)
                    .on_press_maybe(play),
                button(text("Up").size(11)).style(action).on_press_maybe(
                    (!busy && index > 0).then(|| Message::SourceOperation(
                        Operation::MovePlaylistEntry(playlist.clone(), entry.id.clone(), -1)
                    ))
                ),
                button(text("Down").size(11)).style(action).on_press_maybe(
                    (!busy && index + 1 < snapshot.playlist_entries.len()).then(|| {
                        Message::SourceOperation(Operation::MovePlaylistEntry(
                            playlist.clone(),
                            entry.id.clone(),
                            1,
                        ))
                    })
                ),
                button(text("Remove").size(11))
                    .style(action)
                    .on_press_maybe((!busy).then(|| Message::SourceOperation(
                        Operation::RemovePlaylistEntry(playlist.clone(), entry.id.clone())
                    ))),
            ]
            .spacing(5)
            .align_y(Alignment::Center)
            .into()
        });
    let name = snapshot
        .playlists
        .iter()
        .find(|item| item.id == *playlist)
        .map_or("Playlist", |item| item.name.as_str());
    let header = text(name).font(bold()).size(18);
    if snapshot.playlist_entries.is_empty() {
        return column![header, text("Playlist is empty").size(13).style(muted)].spacing(9);
    }
    column![header, scrollable(column(rows).spacing(4)).height(Fill)].spacing(9)
}

pub fn edit(fields: &[String; 7], busy: bool) -> iced::widget::Column<'_, Message> {
    let labels = [
        "Title",
        "Artist",
        "Album",
        "Track number",
        "Disc number",
        "Genre",
        "Year",
    ];
    let fields = column(labels.into_iter().zip(fields.iter()).enumerate().map(
        |(index, (label, value))| {
            row![
                text(label).size(13).width(95),
                text_input(label, value)
                    .size(13)
                    .on_input(move |value| Message::EditField(index, value))
            ]
            .align_y(Alignment::Center)
            .spacing(10)
            .into()
        },
    ))
    .spacing(7);
    column![
        text("Edit track").font(bold()).size(18),
        scrollable(container(fields).style(panel).padding(10)).height(Fill),
        row![
            button(text("Save").size(13))
                .style(primary)
                .on_press_maybe((!busy).then_some(Message::SaveTrackEdit(false))),
            button(text("Clear overrides").size(13))
                .style(action)
                .on_press_maybe((!busy).then_some(Message::SaveTrackEdit(true))),
            button(text("Cancel").size(13))
                .style(action)
                .on_press(Message::CloseTrackEdit)
        ]
        .spacing(8),
    ]
    .spacing(8)
}
