use std::borrow::Cow;

use iced::{
    Alignment, Color, Element, Fill,
    widget::{button, column, container, image, row, scrollable, text, tooltip},
};

use crate::{
    appearance::{Appearance, Surfaces},
    desktop::{
        Message,
        library_sources::{Snapshot, Sources},
        track_editor_focus::SONGS_SCROLL,
    },
    fonts::TABLE_FIGURES,
    session::Session,
};

use super::{
    Icon, LIBRARY_COMPACT_WIDTH, action, bold, clock, control_opacity,
    ellipsized_label::EllipsizedLabel,
    icon_tinted,
    label_text::replace_embedded_breaks,
    library, panel, track_metadata, track_playlist,
    track_row::{Colors as TrackRowColors, TrackRow},
};

#[derive(Clone, Copy)]
struct TrackRowMetrics {
    inset: f32,
    gap: f32,
    play: f32,
    number: f32,
    artwork: f32,
    duration: f32,
    history: f32,
    action: f32,
}

impl TrackRowMetrics {
    fn for_viewport(width: f32) -> Self {
        if width <= LIBRARY_COMPACT_WIDTH {
            Self {
                inset: 15.0,
                gap: 7.0,
                play: 24.0,
                number: 18.0,
                artwork: 36.0,
                duration: 38.0,
                history: 52.0,
                action: 26.0,
            }
        } else {
            Self {
                inset: 24.0,
                gap: 10.0,
                play: 25.0,
                number: 24.0,
                artwork: 40.0,
                duration: 48.0,
                history: 65.0,
                action: 28.0,
            }
        }
    }
}

fn track_row_control(
    kind: Icon,
    label: &'static str,
    width: f32,
    message: Option<Message>,
    ink: Color,
) -> Element<'static, Message> {
    const HIT_HEIGHT: f32 = 27.0;
    const PLAY_GLYPH: f32 = 13.0;
    const ACTION_GLYPH: f32 = 16.0;
    let playback = matches!(kind, Icon::Play | Icon::Pause);
    let edge = if playback { PLAY_GLYPH } else { ACTION_GLYPH };
    let glyph = icon_tinted(kind, move |_| ink)
        .width(edge)
        .height(edge)
        .opacity(control_opacity(message.is_some()));
    tooltip(
        button(container(glyph).center_x(width).center_y(HIT_HEIGHT))
            .padding(0)
            .style(action)
            .on_press_maybe(message),
        text(label).size(11),
        tooltip::Position::Top,
    )
    .style(panel)
    .into()
}

pub(super) fn view<'a>(
    snapshot: &'a Snapshot,
    session: Option<&Session>,
    cover: Option<&image::Handle>,
    sources: &'a Sources,
    appearance: &Appearance,
    viewport_width: f32,
) -> iced::widget::Column<'a, Message> {
    const ROW_CONTENT_HEIGHT: f32 = 56.0;
    const ARTWORK_EDGE: f32 = 38.0;
    const LIST_TOP_INSET: f32 = 8.0;
    const LIST_BOTTOM_INSET: f32 = 30.0;
    const DIVIDER_OPACITY: f32 = 0.5;
    const STATUS_INSET: f32 = 20.0;
    let surfaces = appearance.surfaces();
    let status = sources.source_tracks_error.as_deref().or_else(|| {
        sources
            .source_tracks_loading()
            .then_some("Loading tracks...")
    });
    if let Some(message) = status {
        return column![
            container(
                text(replace_embedded_breaks(message))
                    .size(13)
                    .color(surfaces.muted)
            )
            .padding(STATUS_INSET)
        ];
    }
    let accent_ink = appearance.accent_ink();
    let row_colors = TrackRowColors::for_appearance(appearance);
    let metrics = TrackRowMetrics::for_viewport(viewport_width);
    let busy = sources.busy();
    let saving_editor = sources.editor_save_pending();
    let adding_track = sources.track_add_pending();
    let playing = session.is_some_and(Session::is_playing);
    let current = session.and_then(Session::current);
    let list = snapshot.tracks.iter().enumerate().map(|(index, track)| {
        let active = current.is_some_and(|current| {
            current
                .indexed_id()
                .map_or_else(|| current.path() == track.path, |id| id == track.id)
        });
        let play = track_row_control(
            if active && playing {
                Icon::Pause
            } else {
                Icon::Play
            },
            if active && playing { "Pause" } else { "Play" },
            metrics.play,
            Some(if active {
                Message::TogglePlayback
            } else {
                Message::PlayIndexed(index)
            }),
            accent_ink,
        );
        let song_detail = track
            .album
            .as_deref()
            .or(track.metadata_overridden.then_some("Edited in Muzo"));
        let artist = match song_detail {
            Some(detail) if !detail.is_empty() => format!("{} - {detail}", track.artist).into(),
            _ => Cow::Borrowed(track.artist.as_str()),
        };
        let metadata = column![
            Element::new(EllipsizedLabel::new(track.title.as_str(), 12.0).font(bold())),
            Element::new(EllipsizedLabel::new(artist, 10.0).color(surfaces.muted)),
        ]
        .spacing(4)
        .width(Fill);
        let artwork_column = metrics.artwork.max(ARTWORK_EDGE);
        let title_gap = metrics.gap + metrics.artwork - artwork_column;
        let track_details = row![
            container(library::small_cover(
                cover
                    .filter(|_| active)
                    .or_else(|| snapshot.covers.get(&track.id)),
                ARTWORK_EDGE,
                surfaces
            ))
            .width(artwork_column),
            metadata,
        ]
        .width(Fill)
        .spacing(title_gap)
        .align_y(Alignment::Center);
        let body = row![
            play,
            container(
                text(index + 1)
                    .font(TABLE_FIGURES)
                    .size(10)
                    .color(surfaces.muted)
            )
            .align_x(Alignment::End)
            .width(metrics.number),
            track_details,
            container(
                text(clock(std::time::Duration::from_secs(
                    track.duration_seconds.into()
                )))
                .font(TABLE_FIGURES)
                .size(10)
                .color(surfaces.muted)
            )
            .align_x(Alignment::End)
            .width(metrics.duration),
            container(
                text(format!(
                    "{} {}",
                    track.plays,
                    if track.plays == 1 { "play" } else { "plays" }
                ))
                .font(TABLE_FIGURES)
                .size(10)
                .color(surfaces.muted)
            )
            .align_x(Alignment::End)
            .width(metrics.history),
            track_row_control(
                Icon::Edit,
                "Edit track",
                metrics.action,
                (!busy && !saving_editor && !adding_track)
                    .then(|| Message::EditTrack(track.clone())),
                surfaces.muted
            ),
            track_row_control(
                Icon::AddToPlaylist,
                "Add to playlist",
                metrics.action,
                (!busy && !saving_editor && !adding_track)
                    .then(|| Message::OpenPlaylistPicker(track.clone())),
                surfaces.muted
            ),
        ]
        .spacing(metrics.gap)
        .align_y(Alignment::Center);
        let mut content = column![container(body).center_y(ROW_CONTENT_HEIGHT)];
        if let Some(editor) = &sources.editing
            && editor.track_id == track.id
            && editor.library_id == track.library_id
        {
            content = content.push(track_metadata::editor(
                editor,
                busy,
                saving_editor,
                surfaces,
            ));
        }
        if let Some(draft) = sources
            .playlist_picker
            .as_ref()
            .filter(|draft| draft.track_id == track.id)
        {
            content = content.push(track_playlist::picker(
                draft,
                &snapshot.playlists,
                busy,
                adding_track,
                surfaces,
            ));
        }
        column![
            Element::new(TrackRow::new(
                container(content).width(Fill).padding([0.0, metrics.inset]),
                row_colors,
                active,
                index % 2 == 1,
            )),
            library::line(Surfaces {
                line: surfaces.line.scale_alpha(DIVIDER_OPACITY),
                ..surfaces
            }),
        ]
        .into()
    });
    if snapshot.tracks.is_empty() {
        return column![
            container(
                text("No tracks yet. Muzo syncs filesystem libraries automatically.")
                    .size(13)
                    .color(surfaces.muted),
            )
            .center(Fill)
        ];
    }
    let mut songs = column(list).padding(iced::Padding {
        top: LIST_TOP_INSET,
        bottom: LIST_BOTTOM_INSET,
        ..Default::default()
    });
    if let Some(message) = &sources.playlist_message {
        songs = songs.push(
            container(text(replace_embedded_breaks(message.as_str())).size(12)).padding([12, 24]),
        );
    }
    let row_height = ROW_CONTENT_HEIGHT + library::DIVIDER_HEIGHT;
    let plain_height =
        LIST_TOP_INSET + LIST_BOTTOM_INSET + row_height * snapshot.tracks.len() as f32;
    let page = crate::desktop::LibraryPage::Songs;
    let list = scrollable(songs)
        .id(SONGS_SCROLL)
        .height(Fill)
        .on_scroll(move |viewport| {
            let extra_height = (viewport.content_bounds().height - plain_height).max(0.0);
            Message::ArtworkViewport(
                page,
                crate::desktop::row_cover::viewport_rows(
                    viewport,
                    row_height,
                    LIST_TOP_INSET,
                    extra_height,
                ),
            )
        });
    column![crate::desktop::row_cover::watch_fit(
        list,
        page,
        snapshot.selected.as_deref(),
        snapshot.tracks.len(),
        plain_height,
    )]
}
