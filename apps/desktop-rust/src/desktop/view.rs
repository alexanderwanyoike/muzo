use std::time::Duration;

use iced::{
    Alignment, Color, Element, Fill, Font, Theme, border, color, font,
    widget::{Space, button, column, container, image, row, scrollable, svg, text, tooltip},
};

use crate::session::Session;

mod appearance;
mod compact;
mod label_text;
mod library;
mod playlist_popup;
mod playlists;
mod sidebar_divider;
mod sidebar_name;
#[cfg(test)]
#[path = "view/sidebar_name_tests.rs"]
mod sidebar_name_tests;
mod sleeve_image;
mod source_search_input;
mod sources;
mod track_metadata;
mod track_playlist;

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

pub(super) use library::library;
pub(super) use library::{settings_button, settings_input};
pub(super) use sources::SOURCE_SEARCH;

fn library_content<'a>(
    sources: &'a Sources,
    dropbox: &'a super::dropbox_panel::Settings,
    preferences: &'a crate::appearance::Appearance,
    session: Option<&'a Session>,
    page: (super::LibraryPage, Option<super::appearance_focus::Choice>),
    cover: Option<&image::Handle>,
    viewport_width: f32,
) -> iced::widget::Column<'a, Message> {
    use super::LibraryPage;
    let (page, focus) = page;
    if page == LibraryPage::Settings {
        return library::settings(preferences, dropbox, sources, session.is_some(), focus);
    }
    if page == LibraryPage::Playlists {
        let Some(snapshot) = sources
            .snapshot
            .as_ref()
            .filter(|snapshot| snapshot.selected_playlist.is_some())
        else {
            return column![container(text("No playlists yet").size(18).font(bold())).center(Fill)];
        };
        return playlists::tracks(
            snapshot,
            sources.busy(),
            session,
            cover,
            preferences.surfaces(),
        );
    }
    if let Some(snapshot) = sources
        .snapshot
        .as_ref()
        .filter(|snapshot| snapshot.selected.is_some())
    {
        sources::tracks(
            snapshot,
            session,
            cover,
            sources,
            preferences.surfaces(),
            viewport_width,
        )
    } else if let Some(session) = session.filter(|session| !session.tracks().is_empty()) {
        let heading = row![
            text("Title").size(12).style(muted).width(Fill),
            text("Artist").size(12).style(muted).width(Fill)
        ]
        .padding([5, 10])
        .spacing(8);
        let list = queue_rows(session);
        column![heading, scrollable(list).height(Fill)].spacing(5)
    } else {
        column![library::empty(preferences.surfaces())]
    }
}

#[cfg(test)]
#[test]
fn it_keeps_the_songs_scroll_area_visible_while_editing_metadata() {
    use iced::{
        Font, Pixels, Rectangle, Renderer, Size, Vector,
        advanced::{
            Layout, layout,
            widget::{Id, Operation, Tree, operation::Scrollable},
        },
    };

    #[derive(Default)]
    struct ScrollAreas(Vec<Id>);
    impl Operation for ScrollAreas {
        fn traverse(&mut self, visit: &mut dyn FnMut(&mut dyn Operation)) {
            visit(self);
        }
        fn scrollable(
            &mut self,
            id: Option<&Id>,
            _: Rectangle,
            _: Rectangle,
            _: Vector,
            _: &mut dyn Scrollable,
        ) {
            self.0.extend(id.cloned());
        }
    }

    let mut sources = Sources::new(None);
    let mut snapshot = super::queue_tests::snapshot(&["source"]);
    snapshot.selected = Some("source".into());
    snapshot.tracks.push(crate::catalogue::IndexedTrack {
        id: "song".into(),
        title: "Song".into(),
        artist: "Artist".into(),
        album: None,
        metadata_overridden: false,
        path: "track.mp3".into(),
        plays: 0,
        duration_seconds: 1,
        library_id: "source".into(),
    });
    sources.snapshot = Some(snapshot);
    sources.editing = Some(Box::new(super::library_sources::TrackEditor {
        track_id: "song".into(),
        library_id: "source".into(),
        fields: Default::default(),
        overridden: false,
        error: None,
        action_focus: None,
    }));
    let dropbox = super::dropbox_panel::Settings::default();
    let preferences = crate::appearance::Appearance::default();
    let mut body: Element<'_, Message> = library_content(
        &sources,
        &dropbox,
        &preferences,
        None,
        (super::LibraryPage::Songs, None),
        None,
        super::windows::LIBRARY_SIZE.width,
    )
    .into();
    let renderer = Renderer::new(Font::DEFAULT, Pixels(16.0));
    let mut tree = Tree::new(&body);
    let widget = body.as_widget_mut();
    let node = widget.layout(
        &mut tree,
        &renderer,
        &layout::Limits::new(Size::ZERO, Size::new(900.0, 550.0)),
    );
    let mut areas = ScrollAreas::default();
    widget.operate(&mut tree, Layout::new(&node), &renderer, &mut areas);

    assert!(areas.0.contains(&"indexed-songs".into()));
}

pub(super) use compact::mini;

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
                    .style(muted)
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
        .on_press(if current {
            Message::TogglePlayback
        } else {
            Message::Select(index)
        })
        .into()
    }))
    .spacing(super::mini_focus::QUEUE_ROW_SPACING)
}

fn clock(duration: Duration) -> String {
    let seconds = duration.as_secs();
    format!("{}:{:02}", seconds / 60, seconds % 60)
}

fn error_line<'a>(error: Option<impl text::IntoFragment<'a>>) -> Element<'a, Message> {
    match error {
        Some(error) => text(error).size(12).color(color!(0xf6aca1)).into(),
        None => Space::new().into(),
    }
}

fn playback_error(
    error: Option<&str>,
    retry: bool,
    focused: bool,
    message: Message,
) -> Element<'_, Message> {
    if retry {
        row![
            error_line(error),
            button(text("Retry track").size(12))
                .padding([2, 6])
                .style(move |theme, status| focus_ring(action(theme, status), theme, focused))
                .on_press(message),
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
        weight: font::Weight::Bold,
        ..Font::with_name("Manrope")
    }
}

pub(super) fn panel(theme: &Theme) -> container::Style {
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
    Disc,
    Music,
    Settings,
    Folder,
    Search,
    Edit,
    AddToPlaylist,
    Close,
    Up,
    Down,
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
            theme.palette().text
        }
    })
    .opacity(control_opacity(active));
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

fn control_opacity(enabled: bool) -> f32 {
    if enabled { 1.0 } else { 0.4 }
}

fn track_play_button(
    kind: Icon,
    label: &'static str,
    message: Option<Message>,
) -> Element<'static, Message> {
    tooltip(
        button(
            icon_tinted(kind, |theme| theme.palette().primary)
                .width(14)
                .height(14),
        )
        .padding([6, 7])
        .style(action)
        .on_press_maybe(message),
        text(label).size(11),
        tooltip::Position::Top,
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
        Icon::Library => "<path d='M15 3h6v6m0-6-7 7M3 21l7-7m-1 7H3v-6'/>",
        Icon::Mini => "<path d='m14 10 7-7m-1 7h-6V4M3 21l7-7m-6 0h6v6'/>",
        Icon::Queue => {
            "<path d='M9 18H3m9-6H3m13-6H3m18 4v8m0-8-5 1'/><circle cx='19' cy='18' r='2'/>"
        }
        Icon::Disc => {
            "<circle cx='12' cy='12' r='9'/><circle cx='12' cy='12' r='2'/><path d='M7 10a5 5 0 0 1 3-3m4 10a5 5 0 0 0 3-3'/>"
        }
        Icon::Play => {
            "<path d='M5 5a2 2 0 0 1 3.008-1.728l11.997 6.998a2 2 0 0 1 .003 3.458l-12 7A2 2 0 0 1 5 19z' fill='black' stroke-width='1.4'/>"
        }
        Icon::Pause => {
            "<g fill='black' stroke-width='1.4'><rect x='14' y='3' width='5' height='18' rx='1'/><rect x='5' y='3' width='5' height='18' rx='1'/></g>"
        }
        Icon::Previous => {
            "<path d='M17.971 4.285A2 2 0 0 1 21 6v12a2 2 0 0 1-3.029 1.715l-9.997-5.998a2 2 0 0 1-.003-3.432z' fill='black' stroke-width='1.4'/><path d='M3 20V4' stroke-width='1.4'/>"
        }
        Icon::Next => {
            "<path d='M6.029 4.285A2 2 0 0 0 3 6v12a2 2 0 0 0 3.029 1.715l9.997-5.998a2 2 0 0 0 .003-3.432z' fill='black' stroke-width='1.4'/><path d='M21 4v16' stroke-width='1.4'/>"
        }
        Icon::Settings => {
            "<path d='M4 7h16M4 17h16'/><circle cx='8' cy='7' r='2' fill='white'/><circle cx='16' cy='17' r='2' fill='white'/>"
        }
        Icon::Folder => {
            "<path d='M3 7a2 2 0 0 1 2-2h4l3 3h7a2 2 0 0 1 2 2v7a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2z'/>"
        }
        Icon::Search => "<circle cx='11' cy='11' r='8'/><path d='m21 21-4.3-4.3'/>",
        Icon::AddToPlaylist => "<path d='M4 6h13M4 12h13M4 18h7M18 15v6m-3-3h6'/>",
        Icon::Edit => "<path d='m14 6 4 4M4 20l4-1L20 7l-4-4L4 15z'/>",
        Icon::Up => "<path d='m7 15 5-5 5 5'/>",
        Icon::Down => "<path d='m7 10 5 5 5-5'/>",
        Icon::Close => "<path d='m7 7 10 10m0-10L7 17'/>",
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
