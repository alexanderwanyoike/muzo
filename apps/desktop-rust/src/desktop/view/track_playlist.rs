use iced::{
    Alignment, Element, Fill, Length, border,
    widget::{button, column, container, pick_list, row, scrollable, text},
};

use crate::{
    appearance::Surfaces,
    catalogue::Playlist,
    desktop::{
        Message,
        library_sources::PlaylistPicker,
        playlist_picker_focus::{self, Control},
    },
};

use super::{
    focus_ring,
    library::{bordered_button, settings_button},
    playlist_popup::Choice,
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
        let anchor = pick_list::<Playlist, _, _, _, _, _>(playlists, selected, |playlist| {
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
            if pending {
                style.text_color = surfaces.muted;
                style.handle_color = surfaces.muted;
            }
            style
        });
        let select = Element::new(Choice {
            anchor: anchor.into(),
            popup: draft
                .preview
                .as_deref()
                .filter(|_| !pending)
                .map(|preview| playlist_options(playlists, preview, surfaces)),
            enabled: !pending,
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

fn playlist_options<'a>(
    playlists: &'a [Playlist],
    preview: &str,
    surfaces: Surfaces,
) -> Element<'a, Message> {
    let options = column(playlists.iter().map(|playlist| {
        let highlighted = playlist.id == preview;
        container(
            button(text(&playlist.name).size(11).wrapping(text::Wrapping::None))
                .width(Fill)
                .height(28)
                .padding([5, 8])
                .style(move |theme, status| {
                    let style = iced::overlay::menu::default(theme);
                    let active = highlighted || status == button::Status::Hovered;
                    button::Style {
                        background: active.then_some(style.selected_background),
                        text_color: if active {
                            style.selected_text_color
                        } else {
                            style.text_color
                        },
                        ..Default::default()
                    }
                })
                .on_press(Message::ChooseTrackPlaylist(playlist.id.clone())),
        )
        .id(playlist_picker_focus::option_id(&playlist.id))
        .into()
    }));
    container(
        scrollable(options)
            .id(playlist_picker_focus::OPTIONS)
            .width(Fill)
            .height(Length::Shrink),
    )
    .padding(1)
    .style(move |theme| {
        let style = iced::overlay::menu::default(theme);
        container::Style {
            background: Some(surfaces.raised.into()),
            border: style.border,
            shadow: style.shadow,
            ..Default::default()
        }
    })
    .into()
}
