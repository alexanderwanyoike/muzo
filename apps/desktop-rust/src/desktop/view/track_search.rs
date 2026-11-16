use iced::{
    Alignment, Element,
    widget::{Space, button, pick_list, row, text},
};

use crate::{
    appearance::Surfaces,
    desktop::{LibraryPage, Message, SongScope, library_sources::Sources},
};

use super::{action, library::themed_input};

pub(in crate::desktop) const TRACK_SEARCH: &str = "track-search";

pub(super) fn control(
    sources: &Sources,
    page: LibraryPage,
    surfaces: Surfaces,
) -> Element<'_, Message> {
    let snapshot = sources.snapshot.as_ref();
    let selected = match page {
        LibraryPage::Songs => snapshot.is_some_and(|snapshot| snapshot.selected.is_some()),
        LibraryPage::Playlists => {
            if snapshot.is_none_or(|snapshot| snapshot.selected_playlist.is_none()) {
                return Space::new().into();
            }
            false
        }
        LibraryPage::Settings => return Space::new().into(),
    };
    let scope = sources.track_search.song_scope(snapshot);
    let placeholder = match page {
        LibraryPage::Playlists => "Search playlist tracks",
        _ if scope == SongScope::Source => "Search this source",
        _ => "Search songs",
    };
    let query = sources.track_search.query(page, snapshot);
    let input = themed_input(placeholder, query, surfaces)
        .id(TRACK_SEARCH)
        .width(176)
        .on_input(Message::TrackSearchText);
    let mut controls = row![].spacing(8).align_y(Alignment::Center);
    if selected {
        controls = controls.push(
            pick_list(
                [SongScope::AllSongs, SongScope::Source],
                Some(scope),
                Message::TrackSearchScope,
            )
            .text_size(12),
        );
    }
    controls = controls.push(input);
    if !query.is_empty() {
        controls = controls.push(
            button(text("×").size(16))
                .padding([3, 5])
                .style(action)
                .on_press(Message::TrackSearchText(String::new())),
        );
    }
    controls.into()
}
