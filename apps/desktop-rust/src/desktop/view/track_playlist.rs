use std::fmt::{self, Display};

use iced::{
    Alignment, Element, Fill, Length, border,
    widget::{button, column, container, pick_list, row, scrollable, text},
};

use crate::{
    appearance::Surfaces,
    catalogue::Playlist,
    desktop::{
        Message,
        library_sources::PlaylistPicker,
        playlist_picker_focus::{self, Control},
    },
};

use super::{
    focus_ring,
    label_text::single_paragraph,
    library::{bordered_button, settings_button},
    playlist_popup::Choice,
};

#[derive(Clone, Copy, PartialEq)]
struct PlaylistLabel<'a>(&'a Playlist);

impl Display for PlaylistLabel<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&single_paragraph(self.0.name.as_str()))
    }
}

pub(super) fn picker<'a>(
    draft: &'a PlaylistPicker,
    playlists: &'a [Playlist],
    busy: bool,
    pending: bool,
    surfaces: Surfaces,
) -> Element<'a, Message> {
    let selected = draft
        .playlist
        .as_deref()
        .and_then(|id| playlists.iter().find(|playlist| playlist.id == id));
    let button = |control: Control, label, enabled| {
        let focused = enabled && draft.focus == Some(control);
        container(
            settings_button(label, surfaces)
                .style(move |theme, status| {
                    focus_ring(bordered_button(theme, status, surfaces), theme, focused)
                })
                .on_press_maybe(control.message().filter(|_| enabled)),
        )
        .id(control.id())
    };
    let cancel = button(Control::Cancel, "Cancel", !pending);
    let body = if playlists.is_empty() {
        row![text("No playlists yet.").size(13), cancel]
    } else {
        let focused = !pending && draft.focus == Some(Control::Choice);
        let labels: Vec<_> = playlists.iter().map(PlaylistLabel).collect();
        let anchor = pick_list(labels, selected.map(PlaylistLabel), |playlist| {
            Message::ChooseTrackPlaylist(playlist.0.id.clone())
        })
        .text_size(11)
        .padding([7, 9])
        .style(move |theme, status| {
            let mut style = pick_list::default(theme, status);
            style.background = surfaces.raised.into();
            style.border = if focused {
                border::rounded(4).width(2).color(style.text_color)
            } else {
                border::rounded(4).width(1).color(surfaces.line)
            };
            if pending {
                style.text_color = surfaces.muted;
                style.handle_color = surfaces.muted;
            }
            style
        });
        let select = Element::new(Choice {
            anchor: anchor.into(),
            popup: draft
                .preview
                .as_deref()
                .filter(|_| !pending)
                .map(|preview| playlist_options(playlists, preview, surfaces)),
            enabled: !pending,
        });
        let submit = button(
            Control::Add,
            if pending {
                "Adding..."
            } else {
                "Add to playlist"
            },
            !busy && !pending && selected.is_some(),
        );
        let choice =
            container(column![text("Playlist").size(11).color(surfaces.muted), select,].spacing(5))
                .id(Control::Choice.id());
        row![choice, row![submit, cancel].spacing(8)]
    };
    let mut content = column![body.spacing(8).align_y(Alignment::End).wrap()].spacing(8);
    if let Some(error) = &draft.error {
        content = content.push(text(error).size(12));
    }
    container(content)
        .width(Fill)
        .padding(iced::Padding {
            top: 13.0,
            left: 35.0,
            bottom: 16.0,
            ..Default::default()
        })
        .into()
}

fn playlist_options<'a>(
    playlists: &'a [Playlist],
    preview: &str,
    surfaces: Surfaces,
) -> Element<'a, Message> {
    let options = column(playlists.iter().map(|playlist| {
        let highlighted = playlist.id == preview;
        container(
            button(
                text(single_paragraph(playlist.name.as_str()))
                    .size(11)
                    .wrapping(text::Wrapping::None),
            )
            .width(Fill)
            .height(28)
            .padding([5, 8])
            .style(move |theme, status| {
                let style = iced::overlay::menu::default(theme);
                let background = if highlighted {
                    Some(style.selected_background)
                } else if matches!(status, button::Status::Hovered | button::Status::Pressed) {
                    Some(theme.extended_palette().background.weak.color.into())
                } else {
                    None
                };
                button::Style {
                    background,
                    text_color: if highlighted {
                        style.selected_text_color
                    } else {
                        style.text_color
                    },
                    ..Default::default()
                }
            })
            .on_press(Message::ChooseTrackPlaylist(playlist.id.clone())),
        )
        .id(playlist_picker_focus::option_id(&playlist.id))
        .into()
    }));
    container(
        scrollable(options)
            .id(playlist_picker_focus::OPTIONS)
            .width(Fill)
            .height(Length::Shrink),
    )
    .padding(1)
    .style(move |theme| {
        let style = iced::overlay::menu::default(theme);
        container::Style {
            background: Some(surfaces.raised.into()),
            border: style.border,
            shadow: style.shadow,
            ..Default::default()
        }
    })
    .into()
}

#[cfg(test)]
mod tests {
    use iced::{
        Color, Event, Font, Pixels, Point, Rectangle, Renderer, Size, Theme,
        advanced::{
            Layout, Shell, clipboard, layout, mouse,
            renderer::{Headless, Style},
            widget::Tree,
        },
    };

    use super::*;

    #[test]
    fn it_distinguishes_the_keyboard_preview_from_the_hovered_destination() {
        let playlists = ["First", "Second"].map(|name| Playlist {
            id: name.into(),
            name: name.into(),
        });
        let mut menu = playlist_options(
            &playlists,
            "First",
            crate::appearance::Appearance::default().surfaces(),
        );
        let mut renderer = Renderer::new(Font::DEFAULT, Pixels(16.0));
        let mut tree = Tree::new(&menu);
        let bounds = Rectangle::with_size(Size::new(200.0, 58.0));
        let widget = menu.as_widget_mut();
        let node = widget.layout(
            &mut tree,
            &renderer,
            &layout::Limits::new(bounds.size(), bounds.size()),
        );
        let cursor = mouse::Cursor::Available(Point::new(160.0, 45.0));
        widget.update(
            &mut tree,
            &Event::Window(iced::window::Event::RedrawRequested(
                iced::time::Instant::now(),
            )),
            Layout::new(&node),
            cursor,
            &renderer,
            &mut clipboard::Null,
            &mut Shell::new(&mut Vec::new()),
            &bounds,
        );
        widget.draw(
            &tree,
            &mut renderer,
            &Theme::Light,
            &Style::default(),
            Layout::new(&node),
            cursor,
            &bounds,
        );
        let pixels = renderer.screenshot(Size::new(200, 58), 1.0, Color::WHITE);
        let color = |y: usize| &pixels[(y * 200 + 160) * 4..][..4];

        assert_ne!(color(14), color(45));
    }
}
