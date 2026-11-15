use std::time::Duration;

use iced::{
    Alignment, Color, ContentFit, Element, Fill, Size, Theme, border,
    widget::{Space, button, column, container, image, row, scrollable, slider, stack, text},
};

use crate::{
    appearance::{Appearance, Surfaces},
    session::Session,
    track::Track,
};

use super::{
    Icon, Message, action, bold, clock_label, control_opacity, focus_ring, focusable_icon, icon,
    icon_tinted, playback_error,
};
use crate::desktop::{
    cover::{Cover, SLEEVE_EDGE, SLEEVE_PAD},
    mini_focus::{Control, QUEUE_ROW_HEIGHT, QUEUE_ROW_SPACING, QUEUE_SCROLL},
    row_cover::{Covers, DirectCovers},
};

pub(in crate::desktop) fn mini<'a>(
    session: Option<&'a Session>,
    window: (bool, bool, Size),
    covers: (&Cover, Option<&Covers>, &DirectCovers),
    appearance: &Appearance,
    playback: (Option<&'a str>, bool),
    focus: Option<Control>,
) -> Element<'a, Message> {
    let surfaces = appearance.surfaces();
    let (cover, indexed, direct) = covers;
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
            cover.sleeve(),
            track.is_some(),
            if tight { 170.0 } else { SLEEVE_EDGE as f32 },
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
            queue(
                session,
                (cover.thumbnail(), indexed, direct),
                focus,
                surfaces,
                size.width
            ),
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
    stack![foreground]
        .push_under(underlay)
        .push_under(crate::desktop::material::compact_background(appearance))
        .clip(true)
        .into()
}

fn sleeve(
    thumbnail: Option<&image::Handle>,
    playing: bool,
    edge: f32,
    surfaces: Surfaces,
) -> Element<'static, Message> {
    if let Some(thumbnail) = thumbnail {
        let scale = edge / SLEEVE_EDGE as f32;
        return super::sleeve_image::overhang(thumbnail.clone(), edge, SLEEVE_PAD * scale);
    }
    let inner: Element<'static, Message> = if playing {
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
        clock_label(position, surfaces.muted),
        slider(
            0.0..=duration.as_secs_f32().max(0.1),
            position.as_secs_f32(),
            Message::Seek
        )
        .step(0.1)
        .height(14)
        .style(move |theme, _| slider_style(theme, surfaces, false)),
        clock_label(duration, surfaces.muted),
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
    let glyph = icon_tinted(if playing { Icon::Pause } else { Icon::Play }, |theme| {
        theme.palette().text
    })
    .opacity(control_opacity(available))
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
                    .scale_alpha(control_opacity(available))
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

pub(super) const SLIDER_RAIL_HEIGHT: f32 = 3.0;
pub(super) const VOLUME_THUMB_RADIUS: f32 = 7.0;

pub(super) fn slider_style(theme: &Theme, surfaces: Surfaces, volume: bool) -> slider::Style {
    let primary = theme.palette().primary;
    slider::Style {
        rail: slider::Rail {
            backgrounds: (primary.into(), surfaces.line.into()),
            width: SLIDER_RAIL_HEIGHT,
            border: border::rounded(2),
        },
        handle: slider::Handle {
            shape: slider::HandleShape::Circle {
                radius: if volume { VOLUME_THUMB_RADIUS } else { 0.0 },
            },
            background: primary.into(),
            border_width: 0.0,
            border_color: Color::TRANSPARENT,
        },
    }
}

fn queue<'a>(
    session: Option<&'a Session>,
    covers: (Option<&image::Handle>, Option<&Covers>, &DirectCovers),
    focus: Option<Control>,
    surfaces: Surfaces,
    width: f32,
) -> Element<'a, Message> {
    let (thumbnail, indexed, direct) = covers;
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
            let handle = thumbnail
                .filter(|_| active)
                .or_else(|| indexed.and_then(|covers| covers.get(track.indexed_id()?)))
                .or_else(|| direct.get(track.path()));
            let image: Element<'_, Message> = if let Some(thumbnail) = handle {
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

#[cfg(test)]
mod tests {
    use iced::{
        Font, Pixels, Rectangle, Renderer,
        advanced::{
            Layout, layout, mouse,
            renderer::{Headless, Style},
            widget::Tree,
        },
    };

    use super::*;

    fn paint_play_control(playing: bool, enabled: bool) -> Vec<u8> {
        let surfaces = Appearance::default().surfaces();
        let mut control = play_button(playing, enabled, false, surfaces);
        let mut renderer = Renderer::new(Font::DEFAULT, Pixels(16.0));
        let mut tree = Tree::new(&control);
        let bounds = Rectangle::with_size(Size::new(48.0, 48.0));
        let widget = control.as_widget_mut();
        let node = widget.layout(
            &mut tree,
            &renderer,
            &layout::Limits::new(bounds.size(), bounds.size()),
        );
        widget.draw(
            &tree,
            &mut renderer,
            &Theme::Light,
            &Style::default(),
            Layout::new(&node),
            mouse::Cursor::Unavailable,
            &bounds,
        );
        renderer.screenshot(Size::new(48, 48), 1.0, Color::WHITE)
    }

    #[test]
    fn it_fades_the_disabled_mini_play_disc_and_glyph() {
        let active = paint_play_control(false, true);
        let disabled = paint_play_control(false, false);
        let red_at = |image: &[u8], x: usize, y: usize| f32::from(image[(y * 48 + x) * 4]);
        let active_circle = red_at(&active, 24, 7);
        let disabled_circle = red_at(&disabled, 24, 7);
        let expected = 255.0 - (255.0 - active_circle) * 0.4;
        assert!((disabled_circle - expected).abs() <= 2.0);
        assert!(red_at(&disabled, 24, 24) > red_at(&active, 24, 24));
    }

    #[test]
    fn it_paints_both_wide_pause_bars_in_the_mini_player() {
        let pixels = paint_play_control(true, true);
        let centre = pixels.chunks_exact(48 * 4).nth(24).unwrap();
        let dark = centre.chunks_exact(4).filter(|pixel| pixel[0] < 80).count();
        assert!(
            dark >= 10,
            "expected two filled pause bars, found {dark} dark pixels"
        );
    }
}
