use std::time::Duration;

use iced::{
    Alignment, Color, ContentFit, Element, Fill, Shadow, Size, Theme, Vector, border,
    widget::{Space, button, column, container, image, row, scrollable, slider, stack, text},
};

use crate::{appearance::Surfaces, session::Session, track::Track};

use super::{
    Icon, Message, action, bold, clock, focus_ring, focusable_icon, icon, icon_tinted,
    playback_error,
};
use crate::desktop::{
    cover::Cover,
    mini_focus::{Control, QUEUE_ROW_HEIGHT, QUEUE_ROW_SPACING, QUEUE_SCROLL},
};

pub(in crate::desktop) fn mini<'a>(
    session: Option<&'a Session>,
    window: (bool, bool, Size),
    cover: &Cover,
    surfaces: Surfaces,
    playback: (Option<&'a str>, bool),
    focus: Option<Control>,
) -> Element<'a, Message> {
    let (queue_open, queue_pending, size) = window;
    let (error, retry) = playback;
    let header = container(
        row![
            text("Muzo").size(11).font(bold()).color(surfaces.muted),
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
            focusable_icon(
                Icon::Library,
                "Return to library",
                Some(Message::ClickMiniControl(Control::Library)),
                focus == Some(Control::Library),
            ),
        ]
        .align_y(Alignment::Center),
    )
    .padding([0, 18])
    .center_y(50)
    .style(move |_| container::Style::default().background(surfaces.chrome.scale_alpha(0.05)));

    let track = session.and_then(Session::current);
    let tight = size.height <= 520.0;
    let mut metadata = column![
        text(track.map_or("Not playing", Track::title))
            .size(if tight { 16 } else { 18 })
            .font(bold())
            .wrapping(text::Wrapping::None),
        text(track.and_then(Track::artist).unwrap_or_else(|| {
            if track.is_some() {
                "Unknown Artist"
            } else {
                "Choose a track in your library"
            }
        }))
        .size(12)
        .wrapping(text::Wrapping::None),
    ]
    .align_x(Alignment::Center)
    .spacing(7);
    if let Some(album) = track.and_then(Track::album) {
        metadata = metadata.push(
            text(album)
                .size(11)
                .color(surfaces.muted)
                .wrapping(text::Wrapping::None),
        );
    }

    let playback = column![
        sleeve(
            cover.thumbnail(),
            track.is_some(),
            if tight { 170.0 } else { 240.0 },
            surfaces
        ),
        Space::new().height(if tight { 10 } else { 16 }),
        metadata,
        Space::new().height(if tight { 14 } else { 20 }),
        transport(session, retry, focus, surfaces, tight),
        playback_error(
            error,
            retry,
            focus == Some(Control::Retry),
            Message::ClickMiniControl(Control::Retry)
        ),
    ]
    .align_x(Alignment::Center)
    .width(Fill);
    let inline = if queue_open && size.width <= 680.0 {
        18.0
    } else {
        28.0
    };
    let body = scrollable(
        container(column![
            Space::new().height(if tight { 4.0 } else { 14.0 }),
            playback,
        ])
        .padding(iced::Padding {
            left: inline,
            right: inline,
            bottom: 20.0,
            top: 0.0,
        })
        .center_x(Fill),
    )
    .height(Fill)
    .width(Fill);
    let workspace = if queue_open {
        row![
            queue(session, cover.thumbnail(), focus, surfaces, size.width),
            body
        ]
    } else {
        row![body]
    };
    let foreground = container(column![header, workspace.height(Fill)])
        .width(Fill)
        .height(Fill);
    let underlay: Element<'_, Message> = cover.background().map_or_else(
        || Space::new().width(Fill).height(Fill).into(),
        |background| {
            image(background.clone())
                .content_fit(ContentFit::Cover)
                .opacity(0.12)
                .width(Fill)
                .height(Fill)
                .into()
        },
    );
    stack![foreground].push_under(underlay).clip(true).into()
}

fn sleeve(
    thumbnail: Option<&image::Handle>,
    playing: bool,
    edge: f32,
    surfaces: Surfaces,
) -> Element<'static, Message> {
    let inner: Element<'static, Message> = if let Some(thumbnail) = thumbnail {
        image(thumbnail.clone())
            .content_fit(ContentFit::Cover)
            .width(edge - 2.0)
            .height(edge - 2.0)
            .border_radius(7)
            .into()
    } else if playing {
        icon_tinted(Icon::Disc, move |_| surfaces.muted)
            .width(44)
            .height(44)
            .into()
    } else {
        text("Choose some music in your library")
            .size(13)
            .color(surfaces.muted)
            .width(180)
            .align_x(Alignment::Center)
            .into()
    };
    container(inner)
        .center(edge)
        .style(move |_| container::Style {
            background: Some(surfaces.sidebar.into()),
            border: border::rounded(8),
            shadow: Shadow {
                color: Color::BLACK.scale_alpha(0.145),
                offset: Vector::new(0.0, 12.0),
                blur_radius: 30.0,
            },
            ..Default::default()
        })
        .into()
}

fn transport(
    session: Option<&Session>,
    retry: bool,
    focus: Option<Control>,
    surfaces: Surfaces,
    tight: bool,
) -> Element<'static, Message> {
    let (position, duration) = session.map_or((Duration::ZERO, Duration::ZERO), Session::observe);
    let index = session.and_then(Session::current_index);
    let playing = session.is_some_and(Session::is_playing);
    let timeline = row![
        container(text(clock(position)).size(9).color(surfaces.muted)).center_x(27),
        slider(
            0.0..=duration.as_secs_f32().max(0.1),
            position.as_secs_f32(),
            Message::Seek
        )
        .step(0.1)
        .height(14)
        .style(move |theme, _| slider_style(theme, surfaces, false)),
        container(text(clock(duration)).size(9).color(surfaces.muted)).center_x(27),
    ]
    .spacing(7)
    .align_y(Alignment::Center);
    let controls = row![
        focusable_icon(
            Icon::Previous,
            "Previous track",
            index
                .is_some_and(|index| index > 0)
                .then_some(Message::ClickMiniControl(Control::Previous)),
            focus == Some(Control::Previous),
        ),
        play_button(
            playing,
            retry || index.is_some(),
            focus == Some(Control::Playback),
            surfaces
        ),
        focusable_icon(
            Icon::Next,
            "Next track",
            session
                .zip(index)
                .is_some_and(|(session, index)| index + 1 < session.tracks().len())
                .then_some(Message::ClickMiniControl(Control::Next)),
            focus == Some(Control::Next),
        ),
    ]
    .spacing(24)
    .align_y(Alignment::Center);
    let volume = row![
        icon_tinted(Icon::Volume, move |_| surfaces.muted)
            .width(16)
            .height(16),
        slider(
            0.0..=1.0,
            session.map_or(1.0, Session::volume),
            Message::SetMiniVolume
        )
        .width(124)
        .height(17)
        .step(0.01)
        .style(move |theme, _| slider_style(theme, surfaces, true)),
    ]
    .spacing(10)
    .align_y(Alignment::Center);
    column![
        timeline,
        controls,
        container(volume).padding(2).style(move |theme: &Theme| {
            if focus == Some(Control::Volume) {
                container::Style::default()
                    .border(border::rounded(4).width(2).color(theme.palette().primary))
            } else {
                container::Style::default()
            }
        })
    ]
    .spacing(if tight { 10 } else { 18 })
    .align_x(Alignment::Center)
    .into()
}

fn play_button(
    playing: bool,
    available: bool,
    focused: bool,
    surfaces: Surfaces,
) -> Element<'static, Message> {
    let glyph = icon_tinted(
        if playing { Icon::Pause } else { Icon::Play },
        move |theme| {
            theme
                .palette()
                .text
                .scale_alpha(if available { 1.0 } else { 0.4 })
        },
    )
    .width(26)
    .height(26);
    button(container(glyph).center(48))
        .padding(0)
        .style(move |theme, status| {
            let style = button::Style {
                background: Some(
                    tint(
                        surfaces.raised,
                        theme.palette().primary,
                        if status == button::Status::Hovered {
                            0.24
                        } else {
                            0.14
                        },
                    )
                    .into(),
                ),
                border: border::rounded(24),
                ..action(theme, status)
            };
            focus_ring(style, theme, focused)
        })
        .on_press_maybe(available.then_some(Message::ClickMiniControl(Control::Playback)))
        .into()
}

pub(super) fn slider_style(theme: &Theme, surfaces: Surfaces, volume: bool) -> slider::Style {
    let primary = theme.palette().primary;
    slider::Style {
        rail: slider::Rail {
            backgrounds: (primary.into(), surfaces.line.into()),
            width: 3.0,
            border: border::rounded(2),
        },
        handle: slider::Handle {
            shape: slider::HandleShape::Circle {
                radius: if volume { 7.0 } else { 0.0 },
            },
            background: primary.into(),
            border_width: 0.0,
            border_color: Color::TRANSPARENT,
        },
    }
}

fn queue<'a>(
    session: Option<&'a Session>,
    thumbnail: Option<&image::Handle>,
    focus: Option<Control>,
    surfaces: Surfaces,
    width: f32,
) -> Element<'a, Message> {
    let tracks = session.map_or(&[][..], Session::tracks);
    let count = tracks.len();
    let heading = container(
        row![
            icon_tinted(Icon::Queue, move |_| surfaces.muted),
            column![
                text("Playback queue").font(bold()).size(12),
                text(format!(
                    "{count} {}",
                    if count == 1 { "track" } else { "tracks" }
                ))
                .size(10)
                .color(surfaces.muted)
            ]
            .spacing(3)
        ]
        .spacing(12)
        .align_y(Alignment::Center),
    )
    .padding([16, 18]);
    let rows: Element<'_, Message> = if tracks.is_empty() {
        container(
            text("Play a playlist to build a queue.")
                .size(11)
                .color(surfaces.muted),
        )
        .padding([12, 18])
        .into()
    } else {
        let current = session.and_then(Session::current_index);
        let playing = session.is_some_and(Session::is_playing);
        let list = column(tracks.iter().enumerate().map(|(index, track)| {
            let active = current == Some(index);
            let image: Element<'_, Message> = if let Some(thumbnail) = thumbnail.filter(|_| active)
            {
                image(thumbnail.clone())
                    .content_fit(ContentFit::Cover)
                    .width(36)
                    .height(36)
                    .border_radius(3)
                    .into()
            } else {
                container(icon(Icon::Disc))
                    .center(36)
                    .style(move |_| {
                        container::Style::default()
                            .background(surfaces.sidebar)
                            .border(border::rounded(3).color(surfaces.line).width(1))
                    })
                    .into()
            };
            let position: Element<'_, Message> = if active {
                icon_tinted(if playing { Icon::Pause } else { Icon::Play }, move |_| {
                    surfaces.muted
                })
                .width(12)
                .height(12)
                .into()
            } else {
                text((index + 1).to_string())
                    .size(10)
                    .color(surfaces.muted)
                    .into()
            };
            button(
                row![
                    image,
                    column![
                        text(track.title())
                            .font(bold())
                            .size(11)
                            .wrapping(text::Wrapping::None),
                        text(track.artist().unwrap_or(""))
                            .size(10)
                            .color(surfaces.muted)
                            .wrapping(text::Wrapping::None),
                    ]
                    .spacing(3)
                    .width(Fill),
                    position,
                ]
                .spacing(10)
                .align_y(Alignment::Center),
            )
            .width(Fill)
            .height(QUEUE_ROW_HEIGHT)
            .padding(7)
            .style(move |theme, status| {
                let mut style = action(theme, status);
                if active {
                    style.background =
                        Some(tint(surfaces.raised, theme.palette().primary, 0.13).into());
                }
                focus_ring(style, theme, focus == Some(Control::Track(index)))
            })
            .on_press(Message::ClickMiniControl(Control::Track(index)))
            .into()
        }))
        .spacing(QUEUE_ROW_SPACING);
        scrollable(container(list).padding([0, 8]))
            .id(QUEUE_SCROLL)
            .height(Fill)
            .into()
    };
    let width = if width <= 680.0 { 220 } else { 260 };
    row![
        container(column![heading, rows])
            .width(width - 1)
            .height(Fill)
            .style(
                move |_| container::Style::default().background(surfaces.sidebar.scale_alpha(0.88))
            ),
        container(Space::new())
            .width(1)
            .height(Fill)
            .style(move |_| container::Style::default().background(surfaces.line)),
    ]
    .into()
}

pub(super) fn tint(base: Color, overlay: Color, amount: f32) -> Color {
    Color::from_rgb(
        base.r + (overlay.r - base.r) * amount,
        base.g + (overlay.g - base.g) * amount,
        base.b + (overlay.b - base.b) * amount,
    )
}
