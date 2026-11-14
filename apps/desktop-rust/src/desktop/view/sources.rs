use iced::{
    Alignment, Element, Fill,
    widget::{button, column, container, row, scrollable, text, text_input},
};

use crate::{
    desktop::{
        Message,
        library_sources::{Operation, Snapshot, Sources},
    },
    track::Track,
};

use super::{action, bold, muted, panel, selected};

pub fn sidebar(sources: &Sources) -> Element<'_, Message> {
    let query = sources.search.trim().to_lowercase();
    let selected_id = sources
        .snapshot
        .as_ref()
        .and_then(|snapshot| snapshot.selected.as_deref());
    let source_rows = sources
        .snapshot
        .iter()
        .flat_map(|snapshot| &snapshot.libraries)
        .filter(|source| source.name.to_lowercase().contains(&query))
        .map(|source| {
            let current = selected_id == Some(source.id.as_str());
            let label = text(&source.name)
                .size(13)
                .width(Fill)
                .wrapping(text::Wrapping::None);
            button(label)
                .width(Fill)
                .padding([8, 9])
                .style(move |theme, status| {
                    if current {
                        selected(theme, status)
                    } else {
                        action(theme, status)
                    }
                })
                .on_press_maybe(
                    (!sources.busy())
                        .then(|| Message::SourceOperation(Operation::Select(source.id.clone()))),
                )
                .into()
        });
    let mut panel_content = column![
        text("Sources").font(bold()).size(15),
        text_input("Search sources", &sources.search)
            .size(13)
            .on_input(Message::SourceSearch),
        scrollable(column(source_rows).spacing(2)).height(Fill),
        super::playlists::sidebar(sources),
    ]
    .spacing(9);

    if sources.busy() {
        panel_content = panel_content.push(text("Scanning…").size(12).style(muted));
    }
    if let Some(id) = selected_id {
        let controls = if sources.pending_remove.as_deref() == Some(id) {
            row![
                button(text("Remove source").size(11))
                    .on_press(Message::SourceOperation(Operation::Remove(id.into()))),
                button(text("Cancel").size(11)).on_press(Message::CancelSourceRemoval),
            ]
        } else {
            row![
                button(text("Refresh").size(11))
                    .style(action)
                    .on_press_maybe(
                        (!sources.busy())
                            .then(|| Message::SourceOperation(Operation::Refresh(id.into())))
                    ),
                button(text("Remove").size(11))
                    .style(action)
                    .on_press_maybe(
                        (!sources.busy()).then(|| Message::RequestSourceRemoval(id.into()))
                    ),
            ]
        };
        panel_content = panel_content.push(controls.spacing(5));
    }

    container(panel_content)
        .style(panel)
        .padding(12)
        .width(Fill)
        .height(Fill)
        .into()
}

pub fn tracks<'a>(
    snapshot: &'a Snapshot,
    current: Option<&Track>,
    busy: bool,
) -> iced::widget::Column<'a, Message> {
    let columns = row![
        text("Title").size(12).style(muted).width(Fill),
        text("Artist").size(12).style(muted).width(Fill),
        text("Plays").size(12).style(muted).width(38),
        iced::widget::Space::new().width(145)
    ]
    .padding([5, 10])
    .spacing(8);
    let playing_path = current.map(Track::path);
    let list = snapshot.tracks.iter().enumerate().map(|(index, track)| {
        let active = playing_path == Some(track.path.as_path());
        let play = button(
            row![
                text(&track.title)
                    .size(14)
                    .width(Fill)
                    .wrapping(text::Wrapping::None),
                text(&track.artist)
                    .size(13)
                    .style(muted)
                    .width(Fill)
                    .wrapping(text::Wrapping::None),
                text(track.plays).size(12).style(muted).width(38),
            ]
            .align_y(Alignment::Center)
            .spacing(8),
        )
        .style(move |theme, status| {
            if active {
                selected(theme, status)
            } else {
                action(theme, status)
            }
        })
        .width(Fill)
        .padding([8, 10])
        .on_press(if active {
            Message::TogglePlayback
        } else {
            Message::PlayIndexed(index)
        });
        let options = if busy {
            Vec::new()
        } else {
            snapshot.playlists.clone()
        };
        let add =
            iced::widget::pick_list(options, None::<crate::catalogue::Playlist>, |playlist| {
                Message::SourceOperation(Operation::AddToPlaylist(playlist.id, track.id.clone()))
            })
            .placeholder("Add to playlist")
            .text_size(11)
            .width(115);
        row![
            play,
            button(text("Edit").size(11))
                .style(action)
                .on_press_maybe((!busy).then(|| Message::EditTrack(track.clone()))),
            add
        ]
        .align_y(Alignment::Center)
        .spacing(5)
        .into()
    });
    if snapshot.tracks.is_empty() {
        return column![
            container(text("No audio files found.").size(14).style(muted)).center(Fill)
        ];
    }
    column![columns, scrollable(column(list).spacing(2)).height(Fill)].spacing(5)
}
