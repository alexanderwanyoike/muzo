use iced::{
    Alignment, Element, Fill, Length, Theme, border,
    widget::{Space, button, column, container, image, row, scrollable, text},
};

use crate::{
    appearance::{Appearance, Surfaces},
    desktop::{LibraryPage, Message, dropbox_panel, library_sources::Sources, sidebar, updates},
    session::Session,
};

use super::{
    Icon, action, appearance, bold, error_line, icon_tinted,
    label_text::replace_embedded_breaks,
    library_content, library_transport, playback_error, playlists,
    sidebar_divider::Divider,
    sources, tint,
    window_chrome::{PANE_GUTTER, pane_style},
};

const NAV_SUMMARY_HEIGHT: f32 = 33.0;
const NAV_SELECTION_TINT: f32 = 0.16;

pub(in crate::desktop) fn library<'a>(
    sources: &'a Sources,
    dropbox: &'a dropbox_panel::Settings,
    settings: (
        &'a Appearance,
        &'a updates::Status,
        &'a sidebar::Sidebar,
        LibraryPage,
        Option<&'a crate::desktop::appearance_focus::Choice>,
    ),
    session: Option<&'a Session>,
    cover: Option<&'a image::Handle>,
    playback: (Option<&'a str>, bool),
) -> Element<'a, Message> {
    let (preferences, update, sidebar, page, focus) = settings;
    let (error, retry) = playback;
    let surfaces = preferences.surfaces();
    let shell = preferences.theme().palette().background;
    let width = sidebar.visible_width();
    let panes = row![
        container(navigation(sources, page, session, cover, preferences))
            .padding(iced::Padding {
                left: PANE_GUTTER,
                right: PANE_GUTTER - sidebar::SPLITTER_WIDTH,
                top: PANE_GUTTER,
                bottom: PANE_GUTTER,
            })
            .width(width)
            .height(Fill),
        Space::new().width(sidebar::SPLITTER_WIDTH),
        container(
            container(column![
                toolbar(sources, page, surfaces),
                error_line(sources.error.as_deref().map(replace_embedded_breaks)),
                library_content(
                    sources,
                    dropbox,
                    preferences,
                    session,
                    (page, focus),
                    cover,
                    sidebar.viewport_width(),
                )
                .height(Fill),
            ])
            .padding(iced::padding::bottom(PANE_GUTTER))
            .width(Fill)
            .height(Fill)
            .style(move |_| pane_style(surfaces.raised)),
        )
        .padding(iced::Padding {
            left: 0.0,
            ..iced::Padding::new(PANE_GUTTER)
        })
        .width(Fill)
        .height(Fill),
    ]
    .width(Fill)
    .height(Fill);
    let header = column![
        library_transport::view(
            session,
            cover,
            retry,
            surfaces,
            preferences,
            sidebar.viewport_width()
        ),
        playback_error(error, retry, false, Message::RetryTrack),
    ];
    let footer = container(
        row![Space::new().width(Fill), updates::view(update, surfaces)].align_y(Alignment::Center),
    )
    .padding([5, 12])
    .style(move |_| container::Style::default().background(shell));

    container(Element::new(Divider::around(
        header.into(),
        panes.into(),
        footer.into(),
        width,
        (shell, preferences.accent_ink()),
    )))
    .id(crate::desktop::appearance_focus::LIBRARY)
    .height(Fill)
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
        text(format!("{libraries} libraries - {tracks} tracks"))
            .size(11)
            .color(surfaces.muted),
    )
    .center_y(NAV_SUMMARY_HEIGHT)
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
        .style(move |_| pane_style(surfaces.chrome))
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
            Some(tint(surfaces.chrome, theme.palette().primary, NAV_SELECTION_TINT).into())
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
                text(replace_embedded_breaks(title)).size(18).font(bold()),
            ]
            .spacing(3)
        )
        .padding([0, 26])
        .center_y(64)
        .width(Fill),
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
    focus: Option<&crate::desktop::appearance_focus::Choice>,
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
