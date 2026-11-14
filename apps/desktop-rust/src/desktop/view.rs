use std::time::Duration;

use iced::{
    Alignment, Color, Element, Fill, Font, Length, Theme, border, color, font,
    theme::Palette,
    widget::{
        Space, button, column, container, image, row, scrollable, slider, svg, text, tooltip,
    },
};

use crate::{session::Session, track::Track};

use super::Message;

const BACKGROUND: Color = color!(0x202321);
const PANEL: Color = color!(0x292d2a);
const TEXT: Color = color!(0xecefe9);
const MUTED: Color = color!(0xa9b4aa);
const ACCENT: Color = color!(0xbad6aa);
const INK: Color = color!(0x1e2e1e);

pub fn theme() -> Theme {
    Theme::custom(
        "Studio",
        Palette {
            background: BACKGROUND,
            text: TEXT,
            primary: ACCENT,
            success: ACCENT,
            warning: color!(0xf6dca1),
            danger: color!(0xf6aca1),
        },
    )
}

pub fn library<'a>(
    session: Option<&'a Session>,
    cover: Option<&image::Handle>,
    error: Option<&'a str>,
) -> Element<'a, Message> {
    let header = row![
        text("Library").font(bold()).size(24),
        Space::new().width(Fill),
        button(text("Open files").size(14))
            .padding([7, 14])
            .style(primary)
            .on_press_maybe(session.map(|_| Message::OpenFiles)),
    ]
    .align_y(Alignment::Center)
    .spacing(10);

    let body = if let Some(session) = session.filter(|session| !session.tracks().is_empty()) {
        let heading = row![
            text("Title").size(12).color(MUTED).width(Fill),
            text("Artist").size(12).color(MUTED).width(Fill)
        ]
        .padding([5, 10])
        .spacing(8);
        let list = queue_rows(session);
        column![heading, scrollable(list).height(Fill)].spacing(5)
    } else {
        column![
            container(
                text("Open a local audio file to start listening.")
                    .size(14)
                    .color(MUTED)
            )
            .center(Fill)
        ]
    };

    container(
        column![
            header,
            body.height(Fill),
            error_line(error),
            container(
                row![
                    artwork(cover, 54.0),
                    metadata(session.and_then(Session::current), 15),
                    column![
                        row![
                            Space::new().width(Fill),
                            icon_button(Icon::Mini, "Mini player", Some(Message::ShowMini))
                        ],
                        controls(session),
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

pub fn mini<'a>(
    session: Option<&'a Session>,
    queue_open: bool,
    cover: Option<&image::Handle>,
    error: Option<&'a str>,
) -> Element<'a, Message> {
    let header = row![
        icon_button(
            Icon::Library,
            "Return to library",
            Some(Message::ShowLibrary)
        ),
        Space::new().width(Fill),
        icon_button(
            Icon::Queue,
            if queue_open {
                "Hide queue"
            } else {
                "Show queue"
            },
            Some(Message::ToggleQueue)
        ),
    ]
    .align_y(Alignment::Center);
    let track = session.and_then(Session::current);
    let playback = row![
        artwork(cover, 126.0),
        column![metadata(track, 19), controls(session)]
            .spacing(10)
            .width(Fill),
    ]
    .align_y(Alignment::Center)
    .spacing(16);

    let mut content = column![
        header,
        container(playback).width(Fill).padding(12).style(panel),
        error_line(error),
    ]
    .spacing(8);

    if queue_open {
        let queue = session
            .filter(|session| !session.tracks().is_empty())
            .map_or_else(
                || column![text("Queue is empty").size(13).color(MUTED)],
                queue_rows,
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

fn queue_rows(session: &Session) -> iced::widget::Column<'_, Message> {
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
                    .color(MUTED)
                    .width(Fill)
                    .wrapping(text::Wrapping::None),
            ]
            .spacing(8),
        )
        .width(Fill)
        .padding([8, 10])
        .style(move |theme, status| {
            if current {
                selected(theme, status)
            } else {
                action(theme, status)
            }
        })
        .on_press(Message::Select(index))
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
                .color(MUTED)
                .width(Fill)
                .wrapping(text::Wrapping::None),
        );
    }
    if let Some(album) = track.and_then(Track::album) {
        metadata = metadata.push(
            text(album)
                .size(12)
                .color(MUTED)
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

fn controls(session: Option<&Session>) -> iced::widget::Column<'_, Message> {
    let (position, duration) = session.map_or((Duration::ZERO, Duration::ZERO), Session::observe);
    let available = session.is_some_and(|player| player.current().is_some());
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
        text(clock(position)).size(11).color(MUTED),
        Space::new().width(Fill),
        text(clock(duration)).size(11).color(MUTED)
    ];
    let volume = slider(
        0.0..=1.0,
        session.map_or(1.0, Session::volume),
        Message::SetVolume,
    )
    .step(0.01)
    .width(Length::Fixed(60.0));
    let transport = row![
        icon_button(
            Icon::Previous,
            "Previous track",
            has_previous.then_some(Message::Previous)
        ),
        icon_button(
            if playing { Icon::Pause } else { Icon::Play },
            if playing { "Pause" } else { "Play" },
            available.then_some(Message::TogglePlayback)
        ),
        icon_button(Icon::Next, "Next track", has_next.then_some(Message::Next)),
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

fn bold() -> Font {
    Font {
        weight: font::Weight::Semibold,
        ..Font::DEFAULT
    }
}

fn panel(_: &Theme) -> container::Style {
    container::Style::default()
        .background(PANEL)
        .border(border::rounded(10).color(color!(0x3b423c)).width(1))
}

fn action(_: &Theme, status: button::Status) -> button::Style {
    button::Style {
        background: match status {
            button::Status::Hovered | button::Status::Pressed => Some(color!(0x3b443d).into()),
            _ => None,
        },
        text_color: if status == button::Status::Disabled {
            MUTED.scale_alpha(0.48)
        } else {
            TEXT
        },
        border: border::rounded(6),
        ..Default::default()
    }
}

fn selected(theme: &Theme, status: button::Status) -> button::Style {
    button::Style {
        background: Some(color!(0x374338).into()),
        text_color: ACCENT,
        ..action(theme, status)
    }
}

fn primary(theme: &Theme, status: button::Status) -> button::Style {
    button::Style {
        background: Some(
            if status == button::Status::Hovered {
                color!(0xd2e8c7)
            } else {
                ACCENT
            }
            .into(),
        ),
        text_color: INK,
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
    let icon_color = if message.is_some() {
        TEXT
    } else {
        MUTED.scale_alpha(0.4)
    };
    let style = if matches!(icon_kind, Icon::Play | Icon::Pause) && message.is_some() {
        primary
    } else {
        action
    };
    let glyph = icon_tinted(
        icon_kind,
        if matches!(icon_kind, Icon::Play | Icon::Pause) && message.is_some() {
            INK
        } else {
            icon_color
        },
    );
    tooltip(
        button(glyph)
            .padding([6, 8])
            .style(style)
            .on_press_maybe(message),
        text(label).size(12),
        tooltip::Position::FollowCursor,
    )
    .style(panel)
    .into()
}

fn icon(kind: Icon) -> Element<'static, Message> {
    icon_tinted(kind, MUTED).into()
}

fn icon_tinted(kind: Icon, color: Color) -> svg::Svg<'static> {
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
        .style(move |_, _| svg::Style { color: Some(color) })
}
