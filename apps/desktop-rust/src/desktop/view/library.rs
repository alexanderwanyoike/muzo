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
    window_chrome::{PANE_GUTTER, PANE_RADIUS, pane_style},
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
                container(library_content(
                    sources,
                    dropbox,
                    preferences,
                    session,
                    (page, focus),
                    cover,
                    sidebar.viewport_width(),
                ))
                .width(Fill)
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
) -> iced::widget::Scrollable<'a, Message> {
    let surfaces = preferences.surfaces();
    let dropbox_focus = focus.and_then(|choice| match choice {
        crate::desktop::appearance_focus::Choice::Dropbox(control) => Some(control),
        _ => None,
    });
    let body = column![
        appearance::controls(preferences, focus),
        container(settings_section(
            "Dropbox",
            "Choose the folders you want in your music collection.",
            dropbox.view_with_focus(surfaces, dropbox_focus).into(),
            preferences,
        ))
        .id(crate::desktop::settings_scroll::SOURCE_SETUP)
        .width(Fill),
        settings_section(
            "Library folders",
            "Filesystem libraries sync automatically while Muzo is open and when it starts.",
            sources::folder_form(sources, surfaces, preferences.danger_ink()),
            preferences,
        ),
        sources::indexed_sources(sources, preferences, audio, dropbox),
    ]
    .spacing(20)
    .width(Fill)
    .max_width(850);
    scrollable(container(body).padding([24, 28]).center_x(Fill))
        .id(crate::desktop::appearance_focus::SCROLL)
        .height(Fill)
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
    container(column![
        header,
        line(surfaces),
        container(body).padding(20).width(Fill)
    ])
    .width(Fill)
    .style(move |_| {
        pane_style(surfaces.raised)
            .border(border::rounded(PANE_RADIUS).width(1).color(surfaces.line))
    })
    .into()
}

pub(super) const DIVIDER_HEIGHT: f32 = 1.0;

pub(super) fn line(surfaces: Surfaces) -> Element<'static, Message> {
    container(Space::new())
        .height(DIVIDER_HEIGHT)
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
    const BUTTON_HEIGHT: f32 = 31.0;
    let label = container(text(label).size(13).align_y(iced::Center))
        .center_y(BUTTON_HEIGHT)
        .padding([0, 11]);
    button(label)
        .height(BUTTON_HEIGHT)
        .padding(0)
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
#[test]
fn it_centres_the_label_in_a_fixed_height_settings_button() {
    use iced::{
        Font, Pixels, Rectangle, Renderer, Size,
        advanced::{
            Layout, layout, mouse,
            renderer::{Headless, Style},
            widget::Tree,
        },
    };

    let appearance = Appearance::default();
    let surfaces = appearance.surfaces();
    let theme = appearance.theme();
    let mut view: Element<'_, Message> = settings_button("Browse folders", surfaces).into();
    let mut tree = Tree::new(&view);
    let mut renderer = Renderer::new(Font::DEFAULT, Pixels(16.0));
    let widget = view.as_widget_mut();
    let node = widget.layout(
        &mut tree,
        &renderer,
        &layout::Limits::new(Size::ZERO, Size::new(180.0, 40.0)),
    );
    let layout = Layout::new(&node);
    let bounds = layout.bounds();
    widget.draw(
        &tree,
        &mut renderer,
        &theme,
        &Style::default(),
        layout,
        mouse::Cursor::Unavailable,
        &Rectangle::with_size(bounds.size()),
    );
    let size = Size::new(bounds.width.ceil() as u32, bounds.height as u32);
    let pixels = renderer.screenshot(size, 1.0, theme.palette().background);
    let stride = size.width as usize * 4;
    let background = surfaces.raised.into_rgba8()[0];
    let ink: Vec<_> = pixels
        .chunks_exact(stride)
        .enumerate()
        .skip(2)
        .take(size.height as usize - 4)
        .filter(|(_, row)| {
            row[9 * 4..stride - 9 * 4]
                .chunks_exact(4)
                .any(|pixel| pixel[0].abs_diff(background) > 65)
        })
        .map(|(y, _)| y)
        .collect();
    let top = *ink.first().expect("rendered button label");
    let bottom = size.height as usize - 1 - ink.last().unwrap();
    assert!(
        top.abs_diff(bottom) <= 2,
        "Settings button label has {top} px above and {bottom} px below"
    );
}

#[cfg(test)]
#[test]
fn it_draws_a_solid_rounded_panel_for_settings_sections() {
    use iced::{
        Font, Pixels, Rectangle, Renderer, Size,
        advanced::{
            Layout, layout, mouse,
            renderer::{Headless, Style},
            widget::Tree,
        },
    };

    let appearance = Appearance {
        theme: "carbon".into(),
        mode: "dark".into(),
        ..Default::default()
    };
    let theme = appearance.theme();
    let mut view = settings_section(
        "Library folders",
        "Filesystem libraries sync automatically.",
        text("Settings").into(),
        &appearance,
    );
    let mut tree = Tree::new(&view);
    let mut renderer = Renderer::new(Font::DEFAULT, Pixels(16.0));
    let size = Size::new(480, 160);
    let bounds = Rectangle::with_size(Size::new(size.width as f32, size.height as f32));
    let widget = view.as_widget_mut();
    let node = widget.layout(
        &mut tree,
        &renderer,
        &layout::Limits::new(Size::ZERO, bounds.size()),
    );
    widget.draw(
        &tree,
        &mut renderer,
        &theme,
        &Style::default(),
        Layout::new(&node),
        mouse::Cursor::Unavailable,
        &bounds,
    );
    let pixels = renderer.screenshot(size, 1.0, theme.palette().background);
    let pixel = |x: usize, y: usize| {
        let start = (y * size.width as usize + x) * 4;
        &pixels[start..start + 4]
    };
    let fill = appearance.surfaces().raised.into_rgba8();
    assert!(
        (320..400).all(|x| pixel(x, 17) == fill),
        "the Settings header should be solid"
    );
    assert_eq!(
        pixel(3, 2),
        theme.palette().background.into_rgba8(),
        "the rounded panel should leave the corner clear"
    );
}

#[cfg(test)]
#[test]
fn it_opens_songs_at_the_top_after_scrolling_to_source_setup_in_settings() {
    use iced::{
        Font, Pixels, Rectangle, Renderer, Size, Vector,
        advanced::widget::{
            Id, Operation,
            operation::scrollable::{AbsoluteOffset, Scrollable, scroll_to},
        },
    };
    use iced_runtime::{UserInterface, user_interface::Cache};

    use crate::desktop::{
        appearance_focus::SCROLL as SETTINGS_SCROLL,
        queue_tests::{setup, snapshot},
        track_editor_focus::SONGS_SCROLL,
    };

    struct ReadOffset {
        id: Id,
        y: Option<f32>,
    }

    impl Operation for ReadOffset {
        fn traverse(&mut self, visit: &mut dyn FnMut(&mut dyn Operation)) {
            visit(self);
        }

        fn scrollable(
            &mut self,
            id: Option<&Id>,
            _: Rectangle,
            _: Rectangle,
            translation: Vector,
            _: &mut dyn Scrollable,
        ) {
            if id == Some(&self.id) {
                self.y = Some(translation.y);
            }
        }
    }

    const SETTINGS_OFFSET: f32 = 420.0;
    let (mut desktop, _) = setup();
    let mut catalogue = snapshot(&["source"]);
    catalogue.selected = Some("source".into());
    for index in 0..40 {
        catalogue.tracks.push(crate::catalogue::IndexedTrack {
            id: index.to_string(),
            library_id: "source".into(),
            title: format!("Track {index}"),
            artist: String::new(),
            album: None,
            metadata_overridden: false,
            path: "demo.mp3".into(),
            plays: 0,
            duration_seconds: 0,
            file_size: 0,
            file_mtime: 0,
        });
    }
    desktop.sources.snapshot = Some(catalogue);
    let _ = desktop.update(Message::LibraryPage(LibraryPage::Settings));
    let size = Size::new(850.0, 620.0);
    let id = desktop.windows.library_id();
    let mut renderer = Renderer::new(Font::DEFAULT, Pixels(16.0));
    let mut ui = UserInterface::build(desktop.workspace(id), size, Cache::default(), &mut renderer);
    ui.operate(
        &renderer,
        &mut scroll_to(
            SETTINGS_SCROLL.into(),
            AbsoluteOffset {
                x: Some(0.0),
                y: Some(SETTINGS_OFFSET),
            },
        ),
    );
    let mut settings = ReadOffset {
        id: SETTINGS_SCROLL.into(),
        y: None,
    };
    ui.operate(&renderer, &mut settings);
    assert!(settings.y.is_some_and(|offset| offset >= SETTINGS_OFFSET));

    let cache = ui.into_cache();
    let _ = desktop.update(Message::LibraryPage(LibraryPage::Songs));
    let mut ui = UserInterface::build(desktop.workspace(id), size, cache, &mut renderer);
    let mut songs = ReadOffset {
        id: SONGS_SCROLL.into(),
        y: None,
    };
    ui.operate(&renderer, &mut songs);
    assert_eq!(
        songs.y,
        Some(0.0),
        "Songs inherited the Settings scroll position"
    );
}
