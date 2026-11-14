use iced::{
    Alignment, Color, Element, Fill, Theme, border,
    widget::{Space, button, column, container, row, text, tooltip},
};

use crate::appearance::{ACCENTS, Appearance, MATERIALS, MODES, Surfaces};

use super::{Message, action, bold, library};

const DETAILS: [&str; 8] = [
    "Quiet, clear and contemporary",
    "Layered surfaces, inspired by Spotify",
    "Fine matte grain and satin highlights",
    "Soft neutrals with a little warmth",
    "Fine weave and cool graphite",
    "Soft wood grain and warm tones",
    "Smooth ceramic and gentle highlights",
    "Soft translucency and blurred colour",
];

pub fn controls(appearance: &Appearance) -> Element<'static, Message> {
    let surfaces = appearance.surfaces();
    let variants = MATERIALS
        .into_iter()
        .zip(DETAILS)
        .map(|((id, name), detail)| {
            let mut sample = appearance.clone();
            sample.theme = id.into();
            let choice = material_choice(&sample, name, detail, appearance.theme == id);
            container(choice).width(Fill).into()
        })
        .collect::<Vec<_>>();
    let mut slots = variants.into_iter();
    let materials = column((0..3).map(|_| {
        let padded = (0..3).map(|_| {
            slots
                .next()
                .unwrap_or_else(|| Space::new().width(Fill).into())
        });
        row(padded).spacing(14).width(Fill).into()
    }))
    .spacing(14);

    let modes = row(MODES.into_iter().map(|(id, label)| {
        let selected = appearance.mode == id;
        button(container(text(label).size(12)).center_x(68))
            .padding([5, 0])
            .style(move |theme, status| {
                let style = action(theme, status);
                button::Style {
                    background: Some(
                        if selected {
                            surfaces.raised
                        } else {
                            theme.palette().background
                        }
                        .into(),
                    ),
                    text_color: if selected {
                        style.text_color
                    } else {
                        surfaces.muted
                    },
                    border: border::rounded(3),
                    ..style
                }
            })
            .on_press(Message::AppearanceMode(id.into()))
            .into()
    }))
    .spacing(2);
    let modes = column![
        text("Appearance").size(11).color(surfaces.muted),
        container(modes).padding(2).style(move |theme: &Theme| {
            library::framed(theme.palette().background, surfaces, 5)
        }),
    ]
    .spacing(9);
    let accents = column![
        text("Accent colour").size(11).color(surfaces.muted),
        text("Classic").size(10).color(surfaces.muted),
        accent_group(appearance, &ACCENTS[..10], surfaces),
        text("Glow").size(10).color(surfaces.muted),
        accent_group(appearance, &ACCENTS[10..], surfaces),
    ]
    .spacing(9);

    let header = container(
        column![
            text("Appearance").size(13).font(bold()),
            text("Choose a material, appearance and accent colour.")
                .size(11)
                .color(surfaces.muted),
        ]
        .spacing(5),
    )
    .padding([16, 20])
    .width(Fill)
    .style(move |_| container::Style::default().background(surfaces.chrome));

    container(column![
        header,
        library::line(surfaces),
        container(
            column![
                text("Material").size(11).color(surfaces.muted),
                materials,
                row![modes, accents].spacing(40),
            ]
            .spacing(14)
        )
        .padding(20)
        .width(Fill),
    ])
    .style(move |_| library::framed(surfaces.raised, surfaces, 4))
    .into()
}

fn material_choice(
    appearance: &Appearance,
    name: &'static str,
    detail: &'static str,
    selected: bool,
) -> Element<'static, Message> {
    let surfaces = appearance.surfaces();
    let palette = appearance.theme().palette();
    let sidebar = column([
        sample_line(surfaces.line, Fill),
        sample_line(surfaces.line, Fill),
        sample_line(surfaces.line, Fill),
    ])
    .spacing(8);

    let songs = column![
        sample_line(palette.primary, iced::Length::FillPortion(2)),
        sample_line(surfaces.line, Fill),
        sample_line(surfaces.line, Fill),
    ]
    .spacing(8);
    let preview = container(column![
        container(Space::new())
            .width(Fill)
            .height(19)
            .style(move |_| container::Style::default().background(surfaces.chrome)),
        row![
            container(sidebar)
                .padding([11, 6])
                .width(iced::Length::FillPortion(28))
                .height(Fill)
                .style(move |_| container::Style::default().background(surfaces.sidebar)),
            container(songs)
                .padding([11, 12])
                .width(iced::Length::FillPortion(72))
                .height(Fill),
        ]
        .height(Fill),
    ])
    .height(76)
    .width(Fill)
    .clip(true)
    .style(move |_| library::framed(palette.background, surfaces, 5));
    let id = appearance.theme.clone();
    let preview = button(preview)
        .padding(2)
        .width(Fill)
        .style(move |theme, status| {
            let style = action(theme, status);
            button::Style {
                border: border::rounded(7)
                    .color(theme.palette().primary)
                    .width(if selected { 2 } else { 0 }),
                background: None,
                ..style
            }
        })
        .on_press(Message::Material(id));
    column![
        preview,
        text(name).size(12).font(bold()),
        text(detail).size(10).color(surfaces.muted),
    ]
    .spacing(6)
    .width(Fill)
    .into()
}

fn sample_line(color: Color, width: iced::Length) -> Element<'static, Message> {
    container(Space::new())
        .width(width)
        .height(4)
        .style(move |_| {
            container::Style::default()
                .background(color)
                .border(border::rounded(1))
        })
        .into()
}

fn accent_group(
    appearance: &Appearance,
    choices: &'static [(&'static str, &'static str)],
    surfaces: Surfaces,
) -> Element<'static, Message> {
    row(choices.iter().map(|&(id, label)| {
        let mut sample = appearance.clone();
        sample.accent = id.into();
        let color = sample.theme().palette().primary;
        let selected = appearance.accent == id;
        let dot = container(Space::new()).center(19).style(move |_| {
            container::Style::default()
                .background(color)
                .border(border::rounded(10))
        });
        tooltip(
            button(dot)
                .padding(2)
                .style(move |theme, status| button::Style {
                    background: Some(surfaces.raised.into()),
                    border: border::rounded(15)
                        .color(color)
                        .width(if selected { 2 } else { 0 }),
                    ..action(theme, status)
                })
                .on_press(Message::Accent(id.into())),
            text(label).size(11),
            tooltip::Position::Top,
        )
        .into()
    }))
    .spacing(11)
    .align_y(Alignment::Center)
    .into()
}
