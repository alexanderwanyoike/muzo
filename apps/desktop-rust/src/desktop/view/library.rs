use std::time::Duration;

use iced::{
    Alignment, Element, Fill, Length, Theme, border,
    widget::{Space, button, column, container, image, row, scrollable, slider, text},
};

use crate::{
    appearance::{Appearance, Surfaces},
    desktop::{LibraryPage, Message, dropbox_panel, library_sources::Sources, sidebar, updates},
    session::Session,
    track::Track,
};

use super::{
    Icon, action, appearance, bold, clock, compact, error_line, icon_button, icon_tinted,
    library_content, playback_error, playlists, sources,
};

pub(in crate::desktop) fn library<'a>(
    sources: &'a Sources,
    dropbox: &'a dropbox_panel::Settings,
    settings: (
        &'a Appearance,
        &'a updates::Status,
        &'a sidebar::Sidebar,
        LibraryPage,
        Option<crate::desktop::appearance_focus::Choice>,
    ),
    session: Option<&'a Session>,
    cover: Option<&'a image::Handle>,
    playback: (Option<&'a str>, bool),
) -> Element<'a, Message> {
    let (preferences, update, sidebar, page, focus) = settings;
    let (error, retry) = playback;
    let surfaces = preferences.surfaces();
    let view = iced::widget::pane_grid(&sidebar.grid, |_, pane, _| {
        let body: Element<'_, Message> = match pane {
            sidebar::Pane::Sidebar => navigation(sources, page, session, cover, surfaces),
            sidebar::Pane::Content => column![
                toolbar(sources, page, surfaces),
                error_line(sources.error.as_deref()),
                library_content(sources, dropbox, preferences, session, (page, focus), cover)
                    .height(Fill),
            ]
            .width(Fill)
            .height(Fill)
            .into(),
        };
        iced::widget::pane_grid::Content::new(body)
    })
    .style(move |_| iced::widget::pane_grid::Style {
        hovered_region: iced::widget::pane_grid::Highlight {
            background: surfaces.line.into(),
            border: border::rounded(0),
        },
        picked_split: iced::widget::pane_grid::Line {
            color: surfaces.line,
            width: 1.0,
        },
        hovered_split: iced::widget::pane_grid::Line {
            color: surfaces.line,
            width: 1.0,
        },
    })
    .spacing(sidebar::SPLITTER_SPACING)
    .on_resize(9.0, |event| Message::ResizeSidebar(event.ratio))
    .width(Fill)
    .height(Fill);

    column![
        transport(session, cover, retry, surfaces),
        playback_error(error, retry, false, Message::RetryTrack),
        view,
        container(
            row![Space::new().width(Fill), updates::view(update)].align_y(Alignment::Center),
        )
        .padding([0, 12])
        .height(34)
        .center_y(34)
        .style(move |_| container::Style::default().background(surfaces.chrome)),
    ]
    .height(Fill)
    .into()
}

fn transport<'a>(
    session: Option<&'a Session>,
    cover: Option<&image::Handle>,
    retry: bool,
    surfaces: Surfaces,
) -> Element<'a, Message> {
    let active = session.and_then(Session::current);
    let playing = session.is_some_and(Session::is_playing);
    let index = session.and_then(Session::current_index);
    let (position, duration) = session.map_or((Duration::ZERO, Duration::ZERO), Session::observe);
    let controls = row![
        icon_button(
            Icon::Previous,
            "Previous track",
            index.is_some_and(|i| i > 0).then_some(Message::Previous)
        ),
        chrome_play(playing, retry || active.is_some()),
        icon_button(
            Icon::Next,
            "Next track",
            index
                .zip(session)
                .is_some_and(|(i, s)| i + 1 < s.tracks().len())
                .then_some(Message::Next)
        ),
    ]
    .spacing(13)
    .align_y(Alignment::Center);
    let controls = container(controls).center_x(218);

    let timeline = row![
        container(text(clock(position)).size(9).color(surfaces.muted)).center_x(27),
        slider(
            0.0..=duration.as_secs_f32().max(0.1),
            position.as_secs_f32(),
            Message::Seek
        )
        .height(13)
        .step(0.1)
        .style(move |theme, _| compact::slider_style(theme, surfaces, false)),
        container(text(clock(duration)).size(9).color(surfaces.muted)).center_x(27),
    ]
    .spacing(6)
    .align_y(Alignment::Center);
    let display = container(
        row![
            small_cover(cover, 43.0, surfaces),
            column![
                text(active.map_or("Not Playing", Track::title))
                    .font(bold())
                    .size(11)
                    .wrapping(text::Wrapping::None),
                text(active.and_then(Track::artist).unwrap_or("Select a track"))
                    .size(10)
                    .color(surfaces.muted)
                    .wrapping(text::Wrapping::None),
                timeline,
            ]
            .width(Fill)
            .align_x(Alignment::Center)
        ]
        .spacing(12)
        .align_y(Alignment::Center),
    )
    .width(Fill)
    .max_width(610)
    .padding([7, 12])
    .style(move |_| framed(surfaces.raised.scale_alpha(0.70), surfaces, 5));
    let volume = row![
        icon_tinted(Icon::Volume, move |_| surfaces.muted)
            .width(16)
            .height(16),
        slider(
            0.0..=1.0,
            session.map_or(1.0, Session::volume),
            Message::SetVolume
        )
        .width(100)
        .step(0.01)
        .style(move |theme, _| compact::slider_style(theme, surfaces, true)),
    ]
    .spacing(10)
    .align_y(Alignment::Center);
    let volume = container(volume).align_x(Alignment::End).width(170);
    let surface = container(
        row![controls, container(display).center_x(Fill), volume]
            .spacing(20)
            .align_y(Alignment::Center),
    )
    .padding([0, 24])
    .center_y(81)
    .style(move |theme: &Theme| {
        let gradient = iced::gradient::Linear::new(iced::Radians(std::f32::consts::PI))
            .add_stop(
                0.0,
                compact::tint(surfaces.chrome, theme.palette().background, 0.60),
            )
            .add_stop(1.0, surfaces.chrome);
        container::Style::default().background(gradient)
    });
    let mini = row![
        Space::new().width(Fill),
        icon_button(Icon::Mini, "Mini player", Some(Message::ShowMini)),
    ];
    column![
        iced::widget::stack![surface, container(mini).padding([2, 10])],
        line(surfaces),
    ]
    .into()
}

fn chrome_play(playing: bool, enabled: bool) -> Element<'static, Message> {
    let glyph = icon_tinted(
        if playing { Icon::Pause } else { Icon::Play },
        move |theme| {
            theme
                .palette()
                .text
                .scale_alpha(if enabled { 1.0 } else { 0.4 })
        },
    )
    .width(26)
    .height(26);
    button(container(glyph).center(34))
        .padding(0)
        .style(action)
        .on_press_maybe(enabled.then_some(Message::TogglePlayback))
        .into()
}

fn navigation<'a>(
    sources: &'a Sources,
    page: LibraryPage,
    session: Option<&Session>,
    cover: Option<&image::Handle>,
    surfaces: Surfaces,
) -> Element<'a, Message> {
    let libraries = sources
        .snapshot
        .as_ref()
        .map_or(0, |snapshot| snapshot.libraries.len());
    let tracks = sources
        .snapshot
        .as_ref()
        .map_or(0, |snapshot| snapshot.track_counts.values().sum());
    let heading = container(
        column![
            text("Muzo").size(19).font(bold()),
            text(format!("{libraries} libraries - {tracks} tracks"))
                .size(10)
                .color(surfaces.muted),
        ]
        .spacing(6),
    )
    .padding([0, 10]);
    let nav = column![
        nav_item("Songs", Icon::Music, LibraryPage::Songs, page, surfaces),
        nav_item(
            "Playlists",
            Icon::Queue,
            LibraryPage::Playlists,
            page,
            surfaces
        ),
        nav_item(
            "Settings",
            Icon::Settings,
            LibraryPage::Settings,
            page,
            surfaces
        ),
    ]
    .spacing(3);
    let panel: Element<'_, Message> = if page == LibraryPage::Playlists {
        playlists::sidebar(sources, surfaces)
    } else {
        sources::sidebar(sources, surfaces)
    };
    let mut content = column![heading, nav, panel].spacing(18);
    if session.and_then(Session::current).is_some() {
        content = content.push(container(small_cover(cover, 164.0, surfaces)).center_x(Fill));
    }
    container(content)
        .padding([18, 8])
        .width(Fill)
        .height(Fill)
        .style(move |_| container::Style::default().background(surfaces.sidebar))
        .into()
}

fn nav_item(
    label: &'static str,
    icon: Icon,
    page: LibraryPage,
    current: LibraryPage,
    surfaces: Surfaces,
) -> Element<'static, Message> {
    let active = page == current;
    button(
        row![
            icon_tinted(icon, move |theme| if active {
                theme.palette().primary
            } else {
                surfaces.muted
            })
            .width(16)
            .height(16),
            text(label).size(12).font(bold()),
        ]
        .spacing(10)
        .align_y(Alignment::Center),
    )
    .width(Fill)
    .height(33)
    .padding([7, 10])
    .style(move |theme, status| button::Style {
        background: if active {
            Some(surfaces.raised.into())
        } else {
            action(theme, status).background
        },
        ..action(theme, status)
    })
    .on_press(Message::LibraryPage(page))
    .into()
}

fn toolbar(sources: &Sources, page: LibraryPage, surfaces: Surfaces) -> Element<'_, Message> {
    let (eyebrow, title) = match page {
        LibraryPage::Songs => (
            "LIBRARY",
            sources
                .snapshot
                .as_ref()
                .and_then(|snapshot| {
                    let selected = snapshot.selected.as_deref()?;
                    snapshot
                        .libraries
                        .iter()
                        .find(|library| library.id == selected)
                })
                .map_or("Songs", |source| source.name.as_str()),
        ),
        LibraryPage::Playlists => ("COLLECTIONS", "Playlists"),
        LibraryPage::Settings => ("PREFERENCES", "Settings"),
    };
    column![
        container(
            column![
                text(eyebrow).size(9).color(surfaces.muted),
                text(title).size(18).font(bold()),
            ]
            .spacing(3)
        )
        .padding([0, 26])
        .center_y(64)
        .width(Fill)
        .style(move |_| container::Style::default().background(surfaces.raised)),
        line(surfaces),
    ]
    .into()
}

pub(super) fn empty(surfaces: Surfaces) -> Element<'static, Message> {
    container(
        column![
            text("No music yet").font(bold()).size(20),
            text("Add a folder in Settings. Muzo will sync it automatically.")
                .size(13)
                .color(surfaces.muted),
            button(text("Open Settings").size(13))
                .padding([8, 12])
                .style(move |theme, status| bordered_button(theme, status, surfaces))
                .on_press(Message::LibraryPage(LibraryPage::Settings)),
        ]
        .spacing(23)
        .align_x(Alignment::Center),
    )
    .center(Fill)
    .into()
}

pub(super) fn settings<'a>(
    preferences: &'a Appearance,
    dropbox: &'a dropbox_panel::Settings,
    sources: &'a Sources,
    audio: bool,
    focus: Option<crate::desktop::appearance_focus::Choice>,
) -> iced::widget::Column<'a, Message> {
    let surfaces = preferences.surfaces();
    let refresh = sources
        .snapshot
        .as_ref()
        .and_then(|snapshot| snapshot.selected.as_ref())
        .filter(|_| !sources.busy())
        .map(|id| {
            Message::SourceOperation(super::super::library_sources::Operation::Refresh(
                id.clone(),
            ))
        });
    let body = column![
        appearance::controls(preferences, focus),
        container(
            column![
                text("Sources").size(16).font(bold()),
                row![
                    button(text("Add folder").size(13))
                        .style(move |theme, status| bordered_button(theme, status, surfaces))
                        .on_press_maybe((!sources.busy()).then_some(Message::AddFolder)),
                    button(text("Open files").size(13))
                        .style(move |theme, status| bordered_button(theme, status, surfaces))
                        .on_press_maybe(audio.then_some(Message::OpenFiles)),
                    button(text("Refresh").size(13))
                        .style(move |theme, status| bordered_button(theme, status, surfaces))
                        .on_press_maybe(refresh),
                ]
                .spacing(8),
                dropbox.view(),
            ]
            .spacing(12)
        )
        .padding(16)
        .width(Fill)
        .style(move |_| framed(surfaces.raised, surfaces, 5)),
    ]
    .spacing(20)
    .width(Fill)
    .max_width(850);
    column![
        scrollable(container(body).padding([24, 28]).center_x(Fill))
            .id(crate::desktop::appearance_focus::SCROLL)
            .height(Fill)
    ]
}

pub(super) fn line(surfaces: Surfaces) -> Element<'static, Message> {
    container(Space::new())
        .height(1)
        .width(Fill)
        .style(move |_| container::Style::default().background(surfaces.line))
        .into()
}

pub(super) fn small_cover(
    handle: Option<&image::Handle>,
    edge: f32,
    surfaces: Surfaces,
) -> Element<'static, Message> {
    let child: Element<'_, Message> = if let Some(handle) = handle {
        image(handle.clone())
            .width(edge - 2.)
            .height(edge - 2.)
            .content_fit(iced::ContentFit::Cover)
            .border_radius(3)
            .into()
    } else {
        icon_tinted(Icon::Disc, move |_| surfaces.muted)
            .width(edge / 2.)
            .height(edge / 2.)
            .into()
    };
    container(child)
        .center(Length::Fixed(edge))
        .style(move |_| framed(surfaces.sidebar, surfaces, 3))
        .into()
}

pub(super) fn framed(background: iced::Color, surfaces: Surfaces, radius: u32) -> container::Style {
    container::Style::default()
        .background(background)
        .border(border::rounded(radius as f32).width(1).color(surfaces.line))
}

pub(super) fn bordered_button(
    theme: &Theme,
    status: button::Status,
    surfaces: Surfaces,
) -> button::Style {
    button::Style {
        background: Some(surfaces.raised.into()),
        border: border::rounded(5).width(1).color(surfaces.line),
        ..action(theme, status)
    }
}
