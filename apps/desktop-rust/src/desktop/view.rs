use std::time::Duration;

use iced::{
    Alignment, Color, Element, Fill, Font, Length, Theme, border, color, font,
    widget::{
        Space, button, column, container, image, row, scrollable, slider, svg, text, tooltip,
    },
};

use crate::{session::Session, track::Track};

mod appearance;
mod playlists;
mod sources;

use super::{Message, library_sources::Sources};

fn muted(theme: &Theme) -> iced::widget::text::Style {
    let palette = theme.palette();
    let mix = |foreground: f32, background: f32| foreground * 0.79 + background * 0.21;
    iced::widget::text::Style {
        color: Some(Color::from_rgb(
            mix(palette.text.r, palette.background.r),
            mix(palette.text.g, palette.background.g),
            mix(palette.text.b, palette.background.b),
        )),
    }
}

pub fn library<'a>(
    sources: &'a Sources,
    dropbox: &'a super::dropbox_panel::Settings,
    settings: (
        &'a crate::appearance::Appearance,
        &'a super::updates::Status,
    ),
    session: Option<&'a Session>,
    cover: Option<&image::Handle>,
    playback: (Option<&'a str>, bool),
) -> Element<'a, Message> {
    let (preferences, update_status) = settings;
    let (error, retry) = playback;
    let header = row![
        text("Library").font(bold()).size(24),
        Space::new().width(Fill),
        button(text("Settings").size(14))
            .padding([7, 14])
            .style(action)
            .on_press(Message::Dropbox(super::dropbox_panel::Action::Toggle)),
        button(text("Add folder").size(14))
            .padding([7, 14])
            .style(action)
            .on_press_maybe((!sources.busy()).then_some(Message::AddFolder)),
        button(text("Open files").size(14))
            .padding([7, 14])
            .style(primary)
            .on_press_maybe(session.map(|_| Message::OpenFiles)),
    ]
    .align_y(Alignment::Center)
    .spacing(10);

    let body = if dropbox.visible {
        column![appearance::controls(preferences), dropbox.view()].spacing(22)
    } else if let Some((_, _, fields)) = &sources.editing {
        playlists::edit(fields, sources.busy())
    } else if let Some(snapshot) = sources
        .snapshot
        .as_ref()
        .filter(|snapshot| snapshot.selected_playlist.is_some())
    {
        playlists::tracks(snapshot, sources.busy())
    } else if let Some(snapshot) = sources
        .snapshot
        .as_ref()
        .filter(|snapshot| snapshot.selected.is_some())
    {
        sources::tracks(snapshot, session.and_then(Session::current), sources.busy())
    } else if let Some(session) = session.filter(|session| !session.tracks().is_empty()) {
        let heading = row![
            text("Title").size(12).style(muted).width(Fill),
            text("Artist").size(12).style(muted).width(Fill)
        ]
        .padding([5, 10])
        .spacing(8);
        let list = queue_rows(session, None, false);
        column![heading, scrollable(list).height(Fill)].spacing(5)
    } else {
        column![
            container(
                text("Open a local audio file to start listening.")
                    .size(14)
                    .style(muted)
            )
            .center(Fill)
        ]
    };

    container(
        column![
            header,
            row![
                column![
                    sources::sidebar(sources),
                    super::updates::view(update_status)
                ]
                .spacing(5),
                body.width(Fill).height(Fill)
            ]
            .spacing(16),
            error_line(sources.error.as_deref()),
            playback_error(error, retry, false, false),
            container(
                row![
                    artwork(cover, 54.0),
                    metadata(session.and_then(Session::current), 15),
                    column![
                        row![
                            Space::new().width(Fill),
                            icon_button(Icon::Mini, "Mini player", Some(Message::ShowMini))
                        ],
                        controls(session, retry, None, false),
                    ]
                    .width(Length::Fixed(300.0)),
                ]
                .align_y(Alignment::Center)
                .spacing(15)
            )
            .style(panel)
            .padding(12)
            .width(Fill),
        ]
        .spacing(16),
    )
    .padding(20)
    .height(Fill)
    .width(Fill)
    .into()
}

pub(super) fn mini<'a>(
    session: Option<&'a Session>,
    queue: (bool, bool),
    cover: Option<&image::Handle>,
    playback: (Option<&'a str>, bool),
    focus: Option<super::mini_focus::Control>,
) -> Element<'a, Message> {
    use super::mini_focus::Control;
    let (queue_open, queue_pending) = queue;
    let (error, retry) = playback;
    let header = row![
        focusable_icon(
            Icon::Library,
            "Return to library",
            Some(Message::ClickMiniControl(Control::Library)),
            focus == Some(Control::Library),
        ),
        Space::new().width(Fill),
        focusable_icon(
            Icon::Queue,
            if queue_open {
                "Hide queue"
            } else {
                "Show queue"
            },
            (!queue_pending).then_some(Message::ClickMiniControl(Control::Queue)),
            focus == Some(Control::Queue),
        ),
    ]
    .align_y(Alignment::Center);
    let track = session.and_then(Session::current);
    let playback = row![
        artwork(cover, 126.0),
        column![metadata(track, 19), controls(session, retry, focus, true)]
            .spacing(10)
            .width(Fill),
    ]
    .align_y(Alignment::Center)
    .spacing(16);

    let mut content = column![
        header,
        container(playback).width(Fill).padding(12).style(panel),
        playback_error(error, retry, focus == Some(Control::Retry), true),
    ]
    .spacing(8);

    if queue_open {
        let queue = session
            .filter(|session| !session.tracks().is_empty())
            .map_or_else(
                || column![text("Queue is empty").size(13).style(muted)],
                |session| queue_rows(session, focus, true),
            );
        content = content
            .push(text("Queue").font(bold()).size(15))
            .push(scrollable(queue).height(Fill));
    }
    container(content)
        .padding(14)
        .width(Fill)
        .height(Fill)
        .into()
}

fn queue_rows(
    session: &Session,
    focus: Option<super::mini_focus::Control>,
    is_mini: bool,
) -> iced::widget::Column<'_, Message> {
    column(session.tracks().iter().enumerate().map(|(index, track)| {
        let current = session.current_index() == Some(index);
        button(
            row![
                text(track.title())
                    .size(14)
                    .width(Fill)
                    .wrapping(text::Wrapping::None),
                text(track.artist().unwrap_or(""))
                    .size(13)
                    .style(muted)
                    .width(Fill)
                    .wrapping(text::Wrapping::None),
            ]
            .spacing(8),
        )
        .width(Fill)
        .padding([8, 10])
        .style(move |theme, status| {
            let style = if current {
                selected(theme, status)
            } else {
                action(theme, status)
            };
            focus_ring(
                style,
                theme,
                focus == Some(super::mini_focus::Control::Track(index)),
            )
        })
        .on_press(if is_mini {
            Message::ClickMiniControl(super::mini_focus::Control::Track(index))
        } else if current {
            Message::TogglePlayback
        } else {
            Message::Select(index)
        })
        .into()
    }))
    .spacing(2)
}

fn metadata(track: Option<&Track>, size: u32) -> Element<'_, Message> {
    let mut metadata = column![
        text(track.map_or("Not playing", Track::title))
            .font(bold())
            .size(size)
            .width(Fill)
            .wrapping(text::Wrapping::None),
    ]
    .width(Fill)
    .spacing(3);
    if let Some(artist) = track.and_then(Track::artist) {
        metadata = metadata.push(
            text(artist)
                .size(13)
                .style(muted)
                .width(Fill)
                .wrapping(text::Wrapping::None),
        );
    }
    if let Some(album) = track.and_then(Track::album) {
        metadata = metadata.push(
            text(album)
                .size(12)
                .style(muted)
                .width(Fill)
                .wrapping(text::Wrapping::None),
        );
    }
    metadata.into()
}

fn artwork(cover: Option<&image::Handle>, size: f32) -> Element<'static, Message> {
    let box_size = Length::Fixed(size);
    let contents: Element<'_, Message> = match cover {
        Some(art) => image(art.clone())
            .content_fit(iced::ContentFit::Cover)
            .width(box_size)
            .height(box_size)
            .into(),
        None => icon(Icon::Music),
    };
    container(contents)
        .clip(true)
        .center(box_size)
        .style(|_| {
            container::Style::default()
                .background(color!(0x343a35))
                .border(border::rounded(8))
        })
        .into()
}

fn controls(
    session: Option<&Session>,
    retry: bool,
    focus: Option<super::mini_focus::Control>,
    is_mini: bool,
) -> iced::widget::Column<'_, Message> {
    use super::mini_focus::Control;
    let (position, duration) = session.map_or((Duration::ZERO, Duration::ZERO), Session::observe);
    let available = retry || session.is_some_and(|player| player.current().is_some());
    let playing = session.is_some_and(Session::is_playing);
    let index = session.and_then(Session::current_index);
    let has_previous = index.is_some_and(|index| index > 0);
    let has_next = session
        .zip(index)
        .is_some_and(|(player, index)| index + 1 < player.tracks().len());

    let seek = slider(
        0.0..=duration.as_secs_f32().max(0.1),
        position.as_secs_f32(),
        Message::Seek,
    )
    .step(0.1);
    let times = row![
        text(clock(position)).size(11).style(muted),
        Space::new().width(Fill),
        text(clock(duration)).size(11).style(muted)
    ];
    let volume = slider(
        0.0..=1.0,
        session.map_or(1.0, Session::volume),
        Message::SetVolume,
    )
    .step(0.01)
    .width(Length::Fixed(60.0));
    let message = |control, fallback| {
        if is_mini {
            Message::ClickMiniControl(control)
        } else {
            fallback
        }
    };
    let volume = container(volume).padding(2).style(move |theme: &Theme| {
        let style = container::Style::default();
        if focus == Some(Control::Volume) {
            style.border(
                border::rounded(4)
                    .width(2.0)
                    .color(theme.extended_palette().background.base.text),
            )
        } else {
            style
        }
    });
    let transport = row![
        focusable_icon(
            Icon::Previous,
            "Previous track",
            has_previous.then_some(message(Control::Previous, Message::Previous)),
            focus == Some(Control::Previous),
        ),
        focusable_icon(
            if playing { Icon::Pause } else { Icon::Play },
            if playing { "Pause" } else { "Play" },
            available.then_some(message(Control::Playback, Message::TogglePlayback)),
            focus == Some(Control::Playback),
        ),
        focusable_icon(
            Icon::Next,
            "Next track",
            has_next.then_some(message(Control::Next, Message::Next)),
            focus == Some(Control::Next),
        ),
        Space::new().width(Fill),
        icon(Icon::Volume),
        volume,
    ]
    .spacing(3)
    .align_y(Alignment::Center);

    column![seek, times, transport].spacing(3)
}

fn clock(duration: Duration) -> String {
    let seconds = duration.as_secs();
    format!("{}:{:02}", seconds / 60, seconds % 60)
}

fn error_line(error: Option<&str>) -> Element<'_, Message> {
    match error {
        Some(error) => text(error).size(12).color(color!(0xf6aca1)).into(),
        None => Space::new().into(),
    }
}

fn playback_error(
    error: Option<&str>,
    retry: bool,
    focused: bool,
    is_mini: bool,
) -> Element<'_, Message> {
    if retry {
        row![
            error_line(error),
            button(text("Retry track").size(12))
                .padding([2, 6])
                .style(move |theme, status| focus_ring(action(theme, status), theme, focused))
                .on_press(if is_mini {
                    Message::ClickMiniControl(super::mini_focus::Control::Retry)
                } else {
                    Message::RetryTrack
                }),
        ]
        .align_y(Alignment::Center)
        .spacing(5)
        .into()
    } else {
        error_line(error)
    }
}

fn bold() -> Font {
    Font {
        weight: font::Weight::Semibold,
        ..Font::DEFAULT
    }
}

fn panel(theme: &Theme) -> container::Style {
    let palette = theme.extended_palette();
    container::Style::default()
        .background(palette.background.weak.color)
        .border(
            border::rounded(10)
                .color(palette.background.strong.color)
                .width(1),
        )
}
fn action(theme: &Theme, status: button::Status) -> button::Style {
    let palette = theme.extended_palette();
    button::Style {
        background: match status {
            button::Status::Hovered | button::Status::Pressed => {
                Some(palette.background.strong.color.into())
            }
            _ => None,
        },
        text_color: if status == button::Status::Disabled {
            palette.background.base.text.scale_alpha(0.45)
        } else {
            palette.background.base.text
        },
        border: border::rounded(6),
        ..Default::default()
    }
}
fn selected(theme: &Theme, status: button::Status) -> button::Style {
    let pair = theme.extended_palette().primary.weak;
    button::Style {
        background: Some(pair.color.into()),
        text_color: pair.text,
        ..action(theme, status)
    }
}
fn primary(theme: &Theme, status: button::Status) -> button::Style {
    let palette = theme.extended_palette();
    let pair = if status == button::Status::Hovered {
        palette.primary.strong
    } else {
        palette.primary.base
    };
    button::Style {
        background: Some(pair.color.into()),
        text_color: pair.text,
        ..action(theme, status)
    }
}

#[derive(Clone, Copy)]
enum Icon {
    Library,
    Mini,
    Queue,
    Play,
    Pause,
    Previous,
    Next,
    Music,
    Volume,
}

fn icon_button(
    icon_kind: Icon,
    label: &'static str,
    message: Option<Message>,
) -> Element<'static, Message> {
    focusable_icon(icon_kind, label, message, false)
}

fn focusable_icon(
    icon_kind: Icon,
    label: &'static str,
    message: Option<Message>,
    focused: bool,
) -> Element<'static, Message> {
    let prominent = matches!(icon_kind, Icon::Play | Icon::Pause) && message.is_some();
    let active = message.is_some();
    let style = if prominent { primary } else { action };
    let glyph = icon_tinted(icon_kind, move |theme| {
        if prominent {
            theme.extended_palette().primary.base.text
        } else {
            theme
                .palette()
                .text
                .scale_alpha(if active { 1.0 } else { 0.4 })
        }
    });
    tooltip(
        button(glyph)
            .padding([6, 8])
            .style(move |theme, status| focus_ring(style(theme, status), theme, focused && active))
            .on_press_maybe(message),
        text(label).size(12),
        tooltip::Position::FollowCursor,
    )
    .style(panel)
    .into()
}

fn focus_ring(mut style: button::Style, theme: &Theme, focused: bool) -> button::Style {
    if focused {
        style.border.width = 2.0;
        style.border.color = theme.extended_palette().background.base.text;
    }
    style
}

fn icon(kind: Icon) -> Element<'static, Message> {
    icon_tinted(kind, |theme| theme.palette().text.scale_alpha(0.68)).into()
}

fn icon_tinted(kind: Icon, tint: impl Fn(&Theme) -> Color + 'static) -> svg::Svg<'static> {
    let path = match kind {
        Icon::Library => "<path d='M4 10V4h6M20 14v6h-6M4 4l6 6m10 10-6-6'/>",
        Icon::Mini => "<path d='M4 7V4h16v16h-3M4 12h9v8H4z'/>",
        Icon::Queue => "<path d='M4 6h16M4 12h16M4 18h10'/>",
        Icon::Play => "<path d='m8 5 12 7-12 7z' fill='black'/>",
        Icon::Pause => "<path d='M9 5v14m6-14v14' stroke-width='3'/>",
        Icon::Previous => "<path d='M5 5v14m14-14L8 12l11 7z' fill='black'/>",
        Icon::Next => "<path d='M19 5v14M5 5l11 7-11 7z' fill='black'/>",
        Icon::Music => {
            "<path d='M10 17V6l9-2v11M10 17a3 2-20 1 1-6 1 3 2-20 1 1 6-1m9-2a3 2-20 1 1-6 1 3 2-20 1 1 6-1'/>"
        }
        Icon::Volume => "<path d='M3 9h4l5-4v14l-5-4H3zM16 9a4 4 0 0 1 0 6m2-9a8 8 0 0 1 0 12'/>",
    };
    let markup = format!(
        "<svg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 24 24' fill='none' stroke='black' stroke-width='1.7' stroke-linecap='round' stroke-linejoin='round'>{path}</svg>"
    );
    svg(svg::Handle::from_memory(markup.into_bytes()))
        .width(18)
        .height(18)
        .style(move |theme, _| svg::Style {
            color: Some(tint(theme)),
        })
}
