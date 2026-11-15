use iced::{
    Alignment, Color, Element, Fill, Font, Theme, border,
    widget::{
        Space, button, column, container, image, row, scrollable, stack, text, text_input, tooltip,
    },
};

use crate::{
    appearance::{Appearance, Surfaces},
    desktop::{
        Message,
        dropbox_panel::{Action, Settings as DropboxSettings},
        library_sources::{Operation, Snapshot, Sources},
        track_editor_focus::SONGS_SCROLL,
    },
    session::Session,
};

use super::{
    Icon, action, bold, clock, compact, control_opacity,
    ellipsized_label::EllipsizedLabel,
    icon_button, icon_tinted,
    label_text::replace_embedded_breaks,
    library, panel, track_metadata, track_playlist,
    track_row::{Colors as TrackRowColors, TrackRow},
};

pub(in crate::desktop) const SOURCE_SEARCH: &str = "source-search";

#[cfg(test)]
#[test]
fn it_finds_a_library_by_its_location_as_well_as_its_name() {
    let source = crate::catalogue::Library {
        id: "library".into(),
        name: "Records".into(),
        kind: "filesystem".into(),
        location: "/Music/Archive".into(),
    };

    assert!(matches_source(&source, "archive"));
    assert!(matches_source(&source, "records"));
    assert!(!matches_source(&source, "other"));
}

fn matches_source(source: &crate::catalogue::Library, query: &str) -> bool {
    source.name.to_lowercase().contains(query)
        || source
            .location
            .to_string_lossy()
            .to_lowercase()
            .contains(query)
}

pub(super) fn folder_form(sources: &Sources, surfaces: Surfaces) -> Element<'_, Message> {
    let can_edit = sources.folder_editable();
    let can_submit = can_edit && !sources.busy();
    let browse = library::settings_button("Browse", surfaces)
        .on_press_maybe(can_edit.then_some(Message::AddFolder));
    let name = library::settings_input(&sources.folder_name, surfaces)
        .on_input_maybe(can_edit.then_some(Message::FolderName))
        .on_submit_maybe(can_submit.then_some(Message::SubmitFolder));
    let location = library::settings_input(&sources.folder_location, surfaces)
        .on_input_maybe(can_edit.then_some(Message::FolderLocation))
        .on_submit_maybe(can_submit.then_some(Message::SubmitFolder));
    column![
        column![text("Name").size(11).color(surfaces.muted), name].spacing(6),
        column![
            text("Location").size(11).color(surfaces.muted),
            row![location, browse].spacing(8).align_y(Alignment::Center),
        ]
        .spacing(6),
        library::settings_button(
            if sources.folder_save_pending() {
                "Adding…"
            } else {
                "Add library"
            },
            surfaces,
        )
        .on_press_maybe(can_submit.then_some(Message::SubmitFolder)),
    ]
    .spacing(14)
    .into()
}

pub(super) fn indexed_sources<'a>(
    sources: &'a Sources,
    appearance: &Appearance,
    audio: bool,
    dropbox: &'a DropboxSettings,
) -> Element<'a, Message> {
    let surfaces = appearance.surfaces();
    let button_style =
        move |theme: &Theme, status| library::bordered_button(theme, status, surfaces);
    let mut body = column![
        button(text("Open files").size(12))
            .style(button_style)
            .on_press_maybe(audio.then_some(Message::OpenFiles))
    ]
    .spacing(12);
    for source in sources
        .snapshot
        .iter()
        .flat_map(|snapshot| &snapshot.libraries)
    {
        let removing = sources.pending_remove.as_deref() == Some(&source.id);
        let refresh = if source.kind == "filesystem" {
            Some(Message::SourceOperation(Operation::Refresh(
                source.id.clone(),
            )))
        } else if source.kind == "dropbox" && dropbox.can_refresh() {
            Some(Message::Dropbox(Action::SyncLibrary(source.id.clone())))
        } else {
            None
        };
        let remove = if removing {
            Message::SourceOperation(Operation::Remove(source.id.clone()))
        } else {
            Message::RequestSourceRemoval(source.id.clone())
        };
        let mut controls = row![
            button(
                text(if dropbox.refreshing.as_deref() == Some(&source.id) {
                    "Refreshing…"
                } else {
                    "Refresh"
                })
                .size(11)
            )
            .style(button_style)
            .on_press_maybe(refresh.filter(|_| !sources.busy())),
            button(
                text(if removing {
                    "Confirm removal"
                } else {
                    "Remove"
                })
                .size(11)
            )
            .style(button_style)
            .on_press_maybe((!sources.busy()).then_some(remove)),
        ]
        .spacing(8);
        if removing {
            controls = controls.push(
                button(text("Cancel").size(11))
                    .style(button_style)
                    .on_press(Message::CancelSourceRemoval),
            );
        }
        body = body.push(library::line(surfaces)).push(
            row![
                column![
                    text(replace_embedded_breaks(source.name.as_str()))
                        .size(12)
                        .font(bold())
                        .wrapping(text::Wrapping::WordOrGlyph),
                    text(replace_embedded_breaks(source.location.to_string_lossy()))
                        .size(11)
                        .color(surfaces.muted)
                        .wrapping(text::Wrapping::WordOrGlyph),
                ]
                .spacing(4)
                .width(Fill),
                controls,
            ]
            .spacing(14)
            .align_y(Alignment::Center),
        );
    }
    if let Some((_, status)) = &dropbox.refresh_status {
        body = body.push(
            text(replace_embedded_breaks(status.as_str()))
                .size(11)
                .color(surfaces.muted),
        );
    }
    library::settings_section(
        "Indexed sources",
        "Manual refresh is a fallback for recovery. Normal listening does not require it.",
        body.into(),
        appearance,
    )
}

pub fn sidebar<'a>(sources: &'a Sources, appearance: &Appearance) -> Element<'a, Message> {
    let surfaces = appearance.surfaces();
    let query = sources.search.trim().to_lowercase();
    let selected_id = sources
        .snapshot
        .as_ref()
        .and_then(|snapshot| snapshot.selected.as_deref());
    let source_rows: Vec<Element<'_, Message>> = sources
        .snapshot
        .iter()
        .flat_map(|snapshot| &snapshot.libraries)
        .filter(|source| matches_source(source, &query))
        .map(|source| {
            let current = selected_id == Some(source.id.as_str());
            let count = sources
                .snapshot
                .as_ref()
                .and_then(|snapshot| snapshot.track_counts.get(&source.id).copied())
                .map(|count| format!("{count} {}", if count == 1 { "track" } else { "tracks" }));
            row![
                button(
                    row![
                        icon_tinted(Icon::Folder, move |theme| if current {
                            theme.palette().primary
                        } else {
                            surfaces.muted
                        })
                        .width(15)
                        .height(15),
                        Element::new(EllipsizedLabel::new(source.name.as_str(), 11.0).font(bold())),
                        text(count.unwrap_or_default())
                            .size(9)
                            .color(surfaces.muted),
                    ]
                    .spacing(8)
                    .align_y(Alignment::Center)
                )
                .width(Fill)
                .height(32)
                .padding([5, 8])
                .style(move |theme, status| source_button(theme, status, current, surfaces))
                .on_press_maybe(
                    (!sources.busy())
                        .then(|| Message::SourceOperation(Operation::Select(source.id.clone())))
                ),
                icon_button(
                    Icon::Close,
                    "Remove source",
                    (!sources.busy()).then(|| Message::RequestSourceRemoval(source.id.clone()))
                ),
            ]
            .align_y(Alignment::Center)
            .into()
        })
        .collect();
    let libraries = source_count(sources);
    let heading = container(
        row![
            icon_tinted(Icon::Folder, move |_| surfaces.muted)
                .width(14)
                .height(14),
            text("Sources").size(11).font(bold()),
            Space::new().width(Fill),
            text(libraries.map_or_else(String::new, |count| count.to_string()))
                .size(9)
                .color(surfaces.muted),
        ]
        .spacing(8)
        .align_y(Alignment::Center),
    )
    .padding([0, 10])
    .center_y(32);
    let heading = stack![
        crate::desktop::material::chrome(appearance, surfaces, 32),
        heading
    ];
    let matching = source_rows.len();
    let add = button(
        row![
            text("+").size(16).color(surfaces.muted),
            text("Add source").size(11),
        ]
        .spacing(8)
        .align_y(Alignment::Center),
    )
    .padding([5, 11])
    .height(32)
    .width(Fill)
    .style(move |theme, status| button::Style {
        border: border::rounded(0),
        ..action(theme, status)
    })
    .on_press(Message::OpenSourceSettings);
    let add = stack![
        crate::desktop::material::chrome(appearance, surfaces, 32),
        add
    ];
    let mut body = column![heading, library::line(surfaces)];
    if let Some(libraries) = libraries.filter(|count| *count != 0) {
        body = body.push(source_search(sources, surfaces));
        if !sources.search.is_empty() {
            body = body.push(
                container(
                    text(format!("{matching} of {libraries} sources"))
                        .size(10)
                        .color(surfaces.muted),
                )
                .padding([1, 9]),
            );
        }
        body = body.push(library::line(surfaces));
    }
    let list: Element<'_, Message> = match (libraries, matching) {
        (None, _) if sources.error.is_some() => Space::new().into(),
        (None, _) => source_search_empty("Loading sources...", surfaces).into(),
        (Some(0), _) => {
            source_search_empty("No libraries yet. Add one in Settings.", surfaces).into()
        }
        (_, 0) => source_search_empty("No sources match your search.", surfaces).into(),
        _ => column(source_rows).into(),
    };
    body = body.push(scrollable(list).height(Fill));
    if libraries.is_some() && sources.busy() {
        body =
            body.push(container(text("Scanning…").size(10).color(surfaces.muted)).padding([4, 8]));
    }
    if let Some(source) = sources.pending_remove.as_ref().and_then(|id| {
        sources
            .snapshot
            .as_ref()?
            .libraries
            .iter()
            .find(|row| &row.id == id)
    }) {
        body = body.push(
            row![
                button(
                    text(format!(
                        "Remove {}?",
                        replace_embedded_breaks(source.name.as_str())
                    ))
                    .size(10)
                )
                .style(action)
                .on_press(Message::SourceOperation(Operation::Remove(
                    source.id.clone()
                ))),
                button(text("Cancel").size(10))
                    .style(action)
                    .on_press(Message::CancelSourceRemoval),
            ]
            .spacing(3),
        );
    }
    container(body.push(library::line(surfaces)).push(add))
        .style(move |_| library::framed(surfaces.raised, surfaces, 2))
        .width(Fill)
        .height(Fill)
        .into()
}

fn source_count(sources: &Sources) -> Option<usize> {
    sources
        .snapshot
        .as_ref()
        .map(|snapshot| snapshot.libraries.len())
}

#[cfg(test)]
#[test]
fn it_does_not_report_zero_libraries_before_loading_the_catalogue() {
    let mut sources = Sources::new(None);
    assert_eq!(source_count(&sources), None);
    sources.apply(Err("Could not open catalogue".into()));
    assert_eq!(source_count(&sources), None);
}

fn source_search(sources: &Sources, surfaces: Surfaces) -> Element<'_, Message> {
    let input = text_input("Search sources", &sources.search)
        .id(SOURCE_SEARCH)
        .size(11)
        .padding(iced::Padding {
            left: 27.0,
            right: 28.0,
            top: 8.0,
            bottom: 8.0,
        })
        .style(move |theme, status| search_style(theme, status, surfaces))
        .on_input(Message::SourceSearch);
    let glyph = icon_tinted(Icon::Search, move |_| surfaces.muted)
        .width(13)
        .height(13);
    let mut field = stack![source_search_input(input), container(glyph).padding([8, 8])];
    if !sources.search.is_empty() {
        let clear = icon_button(
            Icon::Close,
            "Clear source search",
            Some(Message::ClearSourceSearch),
        );
        field = field.push(container(clear).width(Fill).align_x(Alignment::End));
    }
    container(field).padding(8).into()
}

fn source_search_input(input: iced::widget::TextInput<'_, Message>) -> Element<'_, Message> {
    Element::new(super::source_search_input::SourceSearchInput(input))
}

#[cfg(test)]
#[test]
fn it_clears_the_focused_source_search_when_escape_is_pressed() {
    use iced::advanced::{
        Layout, Shell, clipboard, layout, mouse,
        widget::{Tree, operation::focusable::focus},
    };
    use iced::{Font, Pixels, Rectangle, Renderer, Size, keyboard};

    let input = text_input("Search sources", "Archive")
        .id(SOURCE_SEARCH)
        .on_input(Message::SourceSearch);
    let mut element = source_search_input(input);
    let mut tree = Tree::new(&element);
    let renderer = Renderer::new(Font::DEFAULT, Pixels(16.0));
    let viewport = Rectangle::with_size(Size::new(220.0, 40.0));
    let widget = element.as_widget_mut();
    let node = widget.layout(
        &mut tree,
        &renderer,
        &layout::Limits::new(Size::ZERO, viewport.size()),
    );
    let layout = Layout::new(&node);
    widget.operate(
        &mut tree,
        layout,
        &renderer,
        &mut focus::<()>(SOURCE_SEARCH.into()),
    );
    let event = iced::Event::Keyboard(keyboard::Event::KeyPressed {
        key: keyboard::Key::Named(keyboard::key::Named::Escape),
        modified_key: keyboard::Key::Named(keyboard::key::Named::Escape),
        physical_key: keyboard::key::Physical::Code(keyboard::key::Code::Escape),
        location: keyboard::Location::Standard,
        modifiers: keyboard::Modifiers::empty(),
        text: None,
        repeat: false,
    });
    let mut messages = Vec::new();
    let mut shell = Shell::new(&mut messages);
    widget.update(
        &mut tree,
        &event,
        layout,
        mouse::Cursor::Unavailable,
        &renderer,
        &mut clipboard::Null,
        &mut shell,
        &viewport,
    );

    assert!(shell.is_event_captured());
    assert!(matches!(messages.as_slice(), [Message::ClearSourceSearch]));
}

fn source_search_empty(
    label: &'static str,
    surfaces: Surfaces,
) -> iced::widget::Container<'static, Message> {
    container(text(label).size(11).color(surfaces.muted)).padding([12, 10])
}

fn search_style(
    theme: &Theme,
    status: text_input::Status,
    surfaces: Surfaces,
) -> text_input::Style {
    let mut style = text_input::default(theme, status);
    style.background = surfaces.raised.into();
    style.border = border::rounded(4).width(1).color(surfaces.line);
    style
}

fn source_button(
    theme: &Theme,
    status: button::Status,
    current: bool,
    surfaces: Surfaces,
) -> button::Style {
    let style = action(theme, status);
    if current {
        button::Style {
            background: Some(compact::tint(surfaces.sidebar, theme.palette().primary, 0.13).into()),
            border: border::rounded(0),
            ..style
        }
    } else {
        style
    }
}

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
        const COMPACT_WIDTH: f32 = 950.0;
        if width <= COMPACT_WIDTH {
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

pub fn tracks<'a>(
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
    const TABLE_TEXT: Font = Font::with_name("Muzo Table Text");
    let surfaces = appearance.surfaces();
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
            _ => std::borrow::Cow::Borrowed(track.artist.as_str()),
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
                    .font(TABLE_TEXT)
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
                .font(TABLE_TEXT)
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
                .font(TABLE_TEXT)
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
                    .color(surfaces.muted)
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
    column![scrollable(songs).id(SONGS_SCROLL).height(Fill)]
}
