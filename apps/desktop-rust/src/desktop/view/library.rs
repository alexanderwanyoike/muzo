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
    Icon, action, appearance, bold, clock, compact, control_opacity, error_line, icon_button,
    icon_tinted, label_text::single_paragraph, library_content, playback_error, playlists,
    sidebar_divider::Divider, sources,
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
    let width = sidebar.visible_width();
    let panes = row![
        container(navigation(sources, page, session, cover, preferences))
            .width(width)
            .height(Fill),
        Space::new().width(sidebar::SPLITTER_WIDTH),
        column![
            toolbar(sources, page, surfaces),
            error_line(sources.error.as_deref()),
            library_content(sources, dropbox, preferences, session, (page, focus), cover)
                .height(Fill),
        ]
        .width(Fill)
        .height(Fill),
    ]
    .width(Fill)
    .height(Fill);
    let header = column![
        transport(session, cover, retry, surfaces, preferences),
        playback_error(error, retry, false, Message::RetryTrack),
    ];
    let footer = container(
        row![Space::new().width(Fill), updates::view(update, surfaces)].align_y(Alignment::Center),
    )
    .padding([5, 12])
    .style(move |_| container::Style::default().background(surfaces.chrome));

    container(Element::new(Divider::around(
        header.into(),
        panes.into(),
        footer.into(),
        width,
        (surfaces.line, preferences.accent_ink()),
    )))
    .id(crate::desktop::appearance_focus::LIBRARY)
    .height(Fill)
    .into()
}

fn transport<'a>(
    session: Option<&'a Session>,
    cover: Option<&image::Handle>,
    retry: bool,
    surfaces: Surfaces,
    preferences: &Appearance,
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
    let surface = iced::widget::stack![
        crate::desktop::material::chrome(preferences, surfaces, 81),
        container(
            row![controls, container(display).center_x(Fill), volume]
                .spacing(20)
                .align_y(Alignment::Center)
        )
        .padding([0, 24])
        .center_y(81),
    ];
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
    let glyph = icon_tinted(if playing { Icon::Pause } else { Icon::Play }, |theme| {
        theme.palette().text
    })
    .opacity(control_opacity(enabled))
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
    preferences: &Appearance,
) -> Element<'a, Message> {
    let surfaces = preferences.surfaces();
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
        sources::sidebar(sources, preferences)
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
                text(single_paragraph(title)).size(18).font(bold()),
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
    let body = column![
        appearance::controls(preferences, focus),
        container(settings_section(
            "Dropbox",
            "Choose the folders you want in your music collection.",
            dropbox.view(surfaces).into(),
            preferences,
        ))
            .id(crate::desktop::settings_scroll::SOURCE_SETUP)
            .width(Fill),
        settings_section(
            "Library folders",
            "Add folders here. Filesystem libraries sync automatically while the app is open and reconcile when it starts.",
            sources::folder_form(sources, surfaces),
            preferences,
        ),
        sources::indexed_sources(sources, preferences, audio, dropbox),
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

pub(super) fn settings_section<'a>(
    title: &'static str,
    description: &'static str,
    body: Element<'a, Message>,
    appearance: &Appearance,
) -> Element<'a, Message> {
    let surfaces = appearance.surfaces();
    let header = container(
        column![
            text(title).size(13).font(bold()),
            text(description).size(11).color(surfaces.muted)
        ]
        .spacing(5),
    )
    .padding([16, 20])
    .width(Fill);
    let header = iced::widget::stack![header]
        .push_under(crate::desktop::material::chrome(appearance, surfaces, Fill))
        .clip(true);
    container(column![
        header,
        line(surfaces),
        container(body).padding(20).width(Fill)
    ])
    .width(Fill)
    .style(move |_| framed(surfaces.raised, surfaces, 4))
    .into()
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

pub(in crate::desktop) fn settings_button(
    label: &str,
    surfaces: Surfaces,
) -> button::Button<'_, Message> {
    button(text(label).size(13))
        .height(31)
        .padding([5, 11])
        .style(move |theme, status| bordered_button(theme, status, surfaces))
}

pub(in crate::desktop) fn settings_input(
    value: &str,
    surfaces: Surfaces,
) -> iced::widget::TextInput<'_, Message> {
    use iced::widget::text_input;

    text_input("", value)
        .size(13)
        .padding([6, 9])
        .style(move |theme, status| {
            let mut style = text_input::default(theme, status);
            style.background = surfaces.raised.into();
            style.border.radius = 4.0.into();
            if !matches!(status, text_input::Status::Focused { .. }) {
                style.border.color = surfaces.line;
            }
            style
        })
}

#[cfg(test)]
mod tests {
    use iced::{
        Color, Font, Pixels, Rectangle, Renderer, Size,
        advanced::{
            Layout, layout, mouse,
            renderer::{Headless, Style},
            widget::Tree,
        },
    };

    use super::*;

    fn play_icon_pixels(enabled: bool) -> Vec<u8> {
        let mut button = chrome_play(false, enabled);
        let mut renderer = Renderer::new(Font::DEFAULT, Pixels(16.0));
        let mut tree = Tree::new(&button);
        let widget = button.as_widget_mut();
        let size = Size::new(34, 34);
        let bounds = Rectangle::with_size(Size::new(34.0, 34.0));
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
}
