use iced::{
    Alignment, Element, Fill,
    widget::{Space, button, column, container, image, row, scrollable, text, text_input},
};

use crate::{
    appearance::Surfaces,
    desktop::{
        Message,
        library_sources::{Operation, Snapshot, Sources},
    },
    session::Session,
    track::Track,
};

use super::{
    Icon, action, bold, compact, icon_button, icon_tinted, library, panel, primary,
    track_play_button,
};

pub fn sidebar(sources: &Sources, surfaces: Surfaces) -> Element<'_, Message> {
    let query = sources.playlist_search.trim().to_lowercase();
    let snapshot = sources.snapshot.as_ref();
    let matches = snapshot
        .into_iter()
        .flat_map(|snapshot| &snapshot.playlists)
        .filter(|playlist| playlist.name.to_lowercase().contains(&query));
    let match_count = matches.clone().count();
    let entries = matches.map(|playlist| {
        let active = snapshot.and_then(|snapshot| snapshot.selected_playlist.as_deref())
            == Some(playlist.id.as_str());
        let count = snapshot
            .and_then(|snapshot| snapshot.playlist_counts.get(&playlist.id).copied())
            .map(|count| count.to_string());
        button(
            row![
                icon_tinted(Icon::Queue, move |theme| if active {
                    theme.palette().primary
                } else {
                    surfaces.muted
                })
                .width(15)
                .height(15),
                text(&playlist.name)
                    .size(11)
                    .font(bold())
                    .width(Fill)
                    .wrapping(text::Wrapping::None),
                text(count.unwrap_or_default())
                    .size(9)
                    .color(surfaces.muted),
            ]
            .spacing(8)
            .align_y(Alignment::Center),
        )
        .height(32)
        .padding([5, 8])
        .width(Fill)
        .style(move |theme, status| {
            let mut style = action(theme, status);
            if active {
                style.background =
                    Some(compact::tint(surfaces.sidebar, theme.palette().primary, 0.13).into());
            }
            style
        })
        .on_press_maybe(
            (!sources.busy())
                .then(|| Message::SourceOperation(Operation::SelectPlaylist(playlist.id.clone()))),
        )
        .into()
    });
    let heading = container(
        row![
            icon_tinted(Icon::Queue, move |_| surfaces.muted)
                .width(14)
                .height(14),
            text("Playlists").size(11).font(bold()),
            Space::new().width(Fill),
            text(snapshot.map_or(0, |snapshot| snapshot.playlists.len()))
                .size(9)
                .color(surfaces.muted),
        ]
        .spacing(8)
        .align_y(Alignment::Center),
    )
    .padding([0, 10])
    .center_y(32)
    .style(move |_| container::Style::default().background(surfaces.chrome));
    let clear = (!query.is_empty()).then(|| Message::PlaylistSearch(String::new()));
    let search = container(
        row![
            text_input("Search playlists", &sources.playlist_search)
                .size(11)
                .padding([8, 9])
                .on_input(Message::PlaylistSearch),
            icon_button(Icon::Close, "Clear search", clear),
        ]
        .align_y(Alignment::Center),
    )
    .padding(8);
    let mut rows = column![];
    if !query.is_empty() {
        rows = rows.push(
            container(
                text(format!("{match_count} matches"))
                    .size(10)
                    .color(surfaces.muted),
            )
            .padding([3, 9]),
        );
        if match_count == 0 {
            rows = rows.push(
                container(
                    text("No playlists match your search.")
                        .size(11)
                        .color(surfaces.muted),
                )
                .padding([5, 9]),
            );
        }
    }
    rows = rows.push(column(entries));
    let footer: Element<'_, Message> = if sources.show_playlist_form {
        let can_edit = !sources.playlist_save_pending();
        let can_submit = can_edit && !sources.busy() && !sources.playlist_name.trim().is_empty();
        container(
            column![
                text_input("Playlist name", &sources.playlist_name)
                    .id("new-playlist-name")
                    .size(11)
                    .on_input_maybe(can_edit.then_some(Message::PlaylistName))
                    .on_submit_maybe(can_submit.then_some(Message::CreatePlaylist)),
                row![
                    button(text("Create playlist").size(11))
                        .style(action)
                        .on_press_maybe(can_submit.then_some(Message::CreatePlaylist)),
                    button(text("Cancel").size(11))
                        .style(action)
                        .on_press_maybe(can_edit.then_some(Message::ShowPlaylistForm(false))),
                ]
                .spacing(6)
            ]
            .spacing(6),
        )
        .padding(8)
        .into()
    } else {
        button(
            row![
                text("+").size(16).color(surfaces.muted),
                text("New playlist").size(11)
            ]
            .spacing(8)
            .align_y(Alignment::Center),
        )
        .width(Fill)
        .height(32)
        .padding([5, 11])
        .style(move |theme, status| button::Style {
            background: Some(surfaces.chrome.into()),
            ..action(theme, status)
        })
        .on_press(Message::ShowPlaylistForm(true))
        .into()
    };
    container(column![
        heading,
        library::line(surfaces),
        search,
        library::line(surfaces),
        scrollable(rows).height(Fill),
        library::line(surfaces),
        footer,
    ])
    .width(Fill)
    .height(Fill)
    .style(move |_| library::framed(surfaces.raised, surfaces, 2))
    .into()
}

pub fn tracks<'a>(
    snapshot: &'a Snapshot,
    busy: bool,
    session: Option<&Session>,
    cover: Option<&image::Handle>,
    surfaces: Surfaces,
) -> iced::widget::Column<'a, Message> {
    let Some(playlist) = &snapshot.selected_playlist else {
        return column![];
    };
    let active_entry = session
        .and_then(Session::current)
        .and_then(Track::playlist_entry);
    let playing = session.is_some_and(Session::is_playing);
    let rows = snapshot
        .playlist_entries
        .iter()
        .enumerate()
        .map(|(index, entry)| {
            let active = active_entry == Some(entry.id.as_str());
            let (label, artist, play) =
                entry
                    .track
                    .as_ref()
                    .map_or(("Unavailable track", "", None), |track| {
                        (
                            track.title.as_str(),
                            track.artist.as_str(),
                            Some(Message::PlayPlaylist(entry.id.clone())),
                        )
                    });
            let content = row![
                library::small_cover(
                    cover.filter(|_| active).or_else(|| entry
                        .track
                        .as_ref()
                        .and_then(|track| snapshot.covers.get(&track.id))),
                    38.0,
                    surfaces
                ),
                column![
                    text(label).size(12).wrapping(text::Wrapping::None),
                    text(artist)
                        .size(10)
                        .color(surfaces.muted)
                        .wrapping(text::Wrapping::None),
                ]
                .spacing(4)
                .width(Fill),
                text(
                    entry
                        .track
                        .as_ref()
                        .map_or(String::new(), |track| format!("{} plays", track.plays))
                )
                .size(10)
                .color(surfaces.muted),
                track_play_button(
                    if active && playing {
                        Icon::Pause
                    } else {
                        Icon::Play
                    },
                    if active && playing { "Pause" } else { "Play" },
                    play,
                ),
                icon_button(
                    Icon::Up,
                    "Move up",
                    (!busy && index > 0).then(|| Message::SourceOperation(
                        Operation::MovePlaylistEntry(playlist.clone(), entry.id.clone(), -1)
                    ))
                ),
                icon_button(
                    Icon::Down,
                    "Move down",
                    (!busy && index + 1 < snapshot.playlist_entries.len()).then(|| {
                        Message::SourceOperation(Operation::MovePlaylistEntry(
                            playlist.clone(),
                            entry.id.clone(),
                            1,
                        ))
                    })
                ),
                button(text("Remove").size(10))
                    .style(action)
                    .on_press_maybe((!busy).then(|| Message::SourceOperation(
                        Operation::RemovePlaylistEntry(playlist.clone(), entry.id.clone())
                    ))),
            ]
            .spacing(7)
            .align_y(Alignment::Center);
            column![
                container(content)
                    .padding([0, 8])
                    .center_y(58)
                    .width(Fill)
                    .style(move |theme: &iced::Theme| {
                        let base = theme.palette().background;
                        let background = if active {
                            compact::tint(base, theme.palette().primary, 0.12)
                        } else if index % 2 == 1 {
                            compact::tint(base, surfaces.line, 0.24)
                        } else {
                            base
                        };
                        container::Style::default().background(background)
                    }),
                library::line(surfaces)
            ]
            .into()
        });
    let name = snapshot
        .playlists
        .iter()
        .find(|item| item.id == *playlist)
        .map_or("Playlist", |item| item.name.as_str());
    let count = snapshot.playlist_entries.len();
    let header = row![
        text(name).font(bold()).size(14),
        text(format!(
            "{count} {}",
            if count == 1 { "track" } else { "tracks" }
        ))
        .size(10)
        .color(surfaces.muted),
        Space::new().width(Fill),
        button(text("Play playlist").size(12))
            .padding([6, 11])
            .style(move |theme, status| library::bordered_button(theme, status, surfaces))
            .on_press_maybe(
                snapshot
                    .playlist_entries
                    .iter()
                    .any(|entry| entry.track.is_some())
                    .then(|| Message::StartPlaylist(playlist.clone()))
            ),
    ]
    .spacing(14)
    .align_y(Alignment::Center);
    let list: Element<'_, Message> = if count == 0 {
        text("Playlist is empty")
            .size(13)
            .color(surfaces.muted)
            .into()
    } else {
        scrollable(column(rows)).height(Fill).into()
    };
    column![
        container(column![header, list].spacing(16))
            .padding([26, 24])
            .height(Fill)
    ]
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
