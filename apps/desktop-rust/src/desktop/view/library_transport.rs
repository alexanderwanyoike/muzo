use std::time::Duration;

use iced::{
    Alignment, Color, Element, Fill, Theme,
    widget::{Space, button, column, container, image, row, slider, text, tooltip},
};

use crate::{
    appearance::{Appearance, Surfaces},
    session::Session,
    track::Track,
};

use super::{
    Icon, LIBRARY_COMPACT_WIDTH, Message, action, bold, clock_label, compact, control_opacity,
    ellipsized_label::EllipsizedLabel,
    icon_tinted,
    library::{framed, line, small_cover},
    panel,
};

mod glow;
mod seek_halo;
mod volume;
use glow::Glow;

const TRANSPORT_HEIGHT: f32 = 81.0;
const DISPLAY_MAX_WIDTH: f32 = 610.0;
const PLAY_BUTTON_EDGE: f32 = 38.0;
const PLAY_GLYPH_EDGE: f32 = 26.0;
const SKIP_BUTTON_EDGE: f32 = 30.0;
const SKIP_GLYPH_EDGE: f32 = 18.0;
const MINI_BUTTON_WIDTH: f32 = 28.0;
const MINI_BUTTON_HEIGHT: f32 = 24.0;
const MINI_GLYPH_EDGE: f32 = 16.0;
const SEEK_INPUT_HEIGHT: f32 = 14.0;
const VOLUME_INPUT_HEIGHT: f32 = 16.0;

struct TransportColumns {
    controls: f32,
    volume: f32,
    volume_slider: f32,
    padding: f32,
    gap: f32,
}

impl TransportColumns {
    fn for_viewport(width: f32) -> Self {
        if width <= LIBRARY_COMPACT_WIDTH {
            Self {
                controls: 150.0,
                volume: 112.0,
                volume_slider: 75.0,
                padding: 17.0,
                gap: 14.0,
            }
        } else {
            Self {
                controls: 218.0,
                volume: 170.0,
                volume_slider: 94.0,
                padding: 24.0,
                gap: 20.0,
            }
        }
    }
}

pub(super) fn view<'a>(
    session: Option<&'a Session>,
    cover: Option<&image::Handle>,
    retry: bool,
    surfaces: Surfaces,
    preferences: &Appearance,
    viewport_width: f32,
) -> Element<'a, Message> {
    const COVER_EDGE: f32 = 43.0;
    const COVER_COLUMN: f32 = 46.0;
    const COVER_GAP: f32 = 12.0;
    const CONTROL_GAP: f32 = 17.0;
    const DISPLAY_PADDING: [u16; 2] = [8, 13];
    const LABEL_LINE_HEIGHT: f32 = 1.4;
    const MINI_BUTTON_INSET: [u16; 2] = [4, 12];
    const TIMELINE_GAP: f32 = 7.0;
    let columns = TransportColumns::for_viewport(viewport_width);
    let glow = Glow::for_appearance(preferences);
    let active = session.and_then(Session::current);
    let playing = session.is_some_and(Session::is_playing);
    let index = session.and_then(Session::current_index);
    let (position, duration) = session.map_or((Duration::ZERO, Duration::ZERO), Session::observe);
    let controls = row![
        transport_button(
            Icon::Previous,
            "Previous track",
            index.is_some_and(|i| i > 0).then_some(Message::Previous),
            None,
            action,
        ),
        chrome_play(playing, retry || active.is_some(), glow),
        transport_button(
            Icon::Next,
            "Next track",
            index
                .zip(session)
                .is_some_and(|(i, s)| i + 1 < s.tracks().len())
                .then_some(Message::Next),
            None,
            action,
        ),
    ]
    .spacing(CONTROL_GAP)
    .align_y(Alignment::Center);
    let controls = container(controls).center_x(columns.controls);

    let seek = slider(
        0.0..=duration.as_secs_f32().max(0.1),
        position.as_secs_f32(),
        Message::Seek,
    )
    .height(SEEK_INPUT_HEIGHT)
    .step(0.1)
    .style(move |theme, _| compact::slider_style(theme, surfaces, false));
    let seek = match glow {
        Some(glow) => glow.surround_seek(seek.into()),
        None => seek.into(),
    };
    let timeline = row![
        clock_label(position, surfaces.muted),
        seek,
        clock_label(duration, surfaces.muted)
    ]
    .spacing(TIMELINE_GAP)
    .align_y(Alignment::Center);
    let display = container(
        row![
            container(small_cover(cover, COVER_EDGE, surfaces)).width(COVER_COLUMN),
            column![
                Element::new(
                    EllipsizedLabel::new(active.map_or("Not Playing", Track::title), 11.0)
                        .font(bold())
                        .line_height(LABEL_LINE_HEIGHT)
                        .center_x()
                ),
                Element::new(
                    EllipsizedLabel::new(
                        active.and_then(Track::artist).unwrap_or("Select a track"),
                        10.0
                    )
                    .color(surfaces.muted)
                    .line_height(LABEL_LINE_HEIGHT)
                    .center_x()
                ),
                timeline,
            ]
            .width(Fill)
            .align_x(Alignment::Center)
        ]
        .spacing(COVER_GAP)
        .align_y(Alignment::Center),
    )
    .width(Fill)
    .max_width(DISPLAY_MAX_WIDTH)
    .padding(DISPLAY_PADDING)
    .style(move |_| framed(surfaces.raised.scale_alpha(0.70), surfaces, 5));
    let level = session.map_or(1.0, Session::volume);
    let volume = slider(0.0..=1.0, level, Message::SetVolume)
        .width(columns.volume_slider)
        .height(VOLUME_INPUT_HEIGHT)
        .step(volume::VOLUME_STEP)
        .style(move |theme, _| {
            let style = compact::slider_style(theme, surfaces, true);
            glow.map_or(style, |glow| glow.volume_ink(style))
        });
    let volume = volume::within_thumb_travel(volume, level);
    let volume = match glow {
        Some(glow) => glow.surround_volume(volume, columns.volume_slider, level),
        None => volume,
    };
    let volume = row![
        icon_tinted(Icon::Volume, move |_| surfaces.muted)
            .width(16)
            .height(16),
        container(volume).padding([0, 2]),
    ]
    .spacing(10)
    .align_y(Alignment::Center);
    let volume = container(volume)
        .align_x(Alignment::End)
        .width(columns.volume);
    let surface = container(
        row![controls, container(display).center_x(Fill), volume]
            .spacing(columns.gap)
            .align_y(Alignment::Center),
    )
    .width(Fill)
    .padding([0.0, columns.padding])
    .center_y(TRANSPORT_HEIGHT)
    .style(move |_| container::Style::default().background(surfaces.chrome));
    let mini = row![
        Space::new().width(Fill),
        transport_button(
            Icon::Mini,
            "Mini player",
            Some(Message::ShowMini),
            Some(surfaces.muted),
            action,
        ),
    ];
    column![
        iced::widget::stack![surface, container(mini).padding(MINI_BUTTON_INSET)],
        line(surfaces),
    ]
    .into()
}

fn chrome_play(playing: bool, enabled: bool, glow: Option<Glow>) -> Element<'static, Message> {
    let (icon, label) = if playing {
        (Icon::Pause, "Pause")
    } else {
        (Icon::Play, "Play")
    };
    let glow = glow.filter(|_| enabled);
    transport_button(
        icon,
        label,
        enabled.then_some(Message::TogglePlayback),
        glow.map(|_| Glow::FOREGROUND),
        move |theme, status| {
            glow.map_or_else(|| action(theme, status), |glow| glow.playback_style(status))
        },
    )
}

fn transport_button(
    kind: Icon,
    label: &'static str,
    message: Option<Message>,
    color: Option<Color>,
    style: impl Fn(&Theme, button::Status) -> button::Style + 'static,
) -> Element<'static, Message> {
    let (glyph_edge, width, height) = match kind {
        Icon::Play | Icon::Pause => (PLAY_GLYPH_EDGE, PLAY_BUTTON_EDGE, PLAY_BUTTON_EDGE),
        Icon::Mini => (MINI_GLYPH_EDGE, MINI_BUTTON_WIDTH, MINI_BUTTON_HEIGHT),
        _ => (SKIP_GLYPH_EDGE, SKIP_BUTTON_EDGE, SKIP_BUTTON_EDGE),
    };
    let glyph = icon_tinted(kind, move |theme| {
        color.unwrap_or_else(|| theme.palette().text)
    })
    .opacity(control_opacity(message.is_some()))
    .width(glyph_edge)
    .height(glyph_edge);
    tooltip(
        button(container(glyph).center_x(width).center_y(height))
            .padding(0)
            .style(style)
            .on_press_maybe(message),
        text(label).size(12),
        if matches!(kind, Icon::Mini) {
            tooltip::Position::FollowCursor
        } else {
            tooltip::Position::Bottom
        },
    )
    .gap(6)
    .style(panel)
    .into()
}

#[cfg(test)]
mod tests {
    use iced::{
        Color, Font, Pixels, Rectangle, Renderer, Size, Theme,
        advanced::{
            Layout, layout, mouse,
            renderer::{Headless, Style},
            widget::Tree,
        },
    };

    use super::*;

    fn play_icon_pixels(enabled: bool) -> Vec<u8> {
        let mut button = chrome_play(false, enabled, None);
        let mut renderer = Renderer::new(Font::DEFAULT, Pixels(16.0));
        let mut tree = Tree::new(&button);
        let widget = button.as_widget_mut();
        let size = Size::new(PLAY_BUTTON_EDGE as u32, PLAY_BUTTON_EDGE as u32);
        let bounds = Rectangle::with_size(Size::new(PLAY_BUTTON_EDGE, PLAY_BUTTON_EDGE));
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
        renderer.screenshot(size, 1.0, Color::WHITE)
    }

    #[test]
    fn it_dims_the_play_glyph_when_no_track_is_selected() {
        let active = play_icon_pixels(true);
        let disabled = play_icon_pixels(false);
        let darkest = |pixels: &[u8]| pixels.chunks_exact(4).map(|color| color[0]).min().unwrap();
        assert!(darkest(&active) < darkest(&disabled));
        assert!(darkest(&disabled) < 255);
    }
    #[test]
    fn it_places_only_the_play_and_pause_tooltips_below_the_library_controls() {
        use iced::{
            Event, Point, Vector,
            advanced::{Shell, clipboard},
        };

        let renderer = Renderer::new(Font::DEFAULT, Pixels(16.0));
        let viewport = Rectangle::with_size(Size::new(240.0, 160.0));
        let origin = Vector::new(90.0, 55.0);
        for (kind, label, below) in [
            (Icon::Play, "Play", true),
            (Icon::Pause, "Pause", true),
            (Icon::Previous, "Previous", false),
            (Icon::Next, "Next", false),
        ] {
            let mut control =
                transport_button(kind, label, Some(Message::TogglePlayback), None, action);
            let mut tree = Tree::new(&control);
            let widget = control.as_widget_mut();
            let node = widget
                .layout(
                    &mut tree,
                    &renderer,
                    &layout::Limits::new(Size::ZERO, viewport.size()),
                )
                .translate(origin);
            let layout = Layout::new(&node);
            let bounds = layout.bounds();
            let cursor = Point::new(bounds.center_x(), bounds.center_y());
            widget.update(
                &mut tree,
                &Event::Mouse(mouse::Event::CursorMoved { position: cursor }),
                layout,
                mouse::Cursor::Available(cursor),
                &renderer,
                &mut clipboard::Null,
                &mut Shell::new(&mut Vec::new()),
                &viewport,
            );
            let mut overlay = widget
                .overlay(&mut tree, layout, &renderer, &viewport, Vector::ZERO)
                .expect("hover should show a tooltip");
            let overlay_layout = overlay.as_overlay_mut().layout(&renderer, viewport.size());
            let tip = Layout::new(&overlay_layout)
                .children()
                .next()
                .expect("tooltip overlay")
                .bounds();
            if below {
                assert!(
                    tip.y > bounds.y + bounds.height,
                    "{label} overlaps its control"
                );
            } else {
                assert!(
                    tip.y < cursor.y,
                    "{label} should keep its cursor-relative tooltip"
                );
            }
        }
    }
}
