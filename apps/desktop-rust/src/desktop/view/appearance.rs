use iced::{
    Alignment, Color, Element, Fill, Shadow, Theme, border,
    widget::{Space, button, column, container, row, text, tooltip},
};

use crate::appearance::{ACCENTS, Appearance, MATERIALS, MODES, Surfaces};

use crate::desktop::appearance_focus::Choice;

use super::{Message, action, bold, focus_ring, library};

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

pub fn controls(appearance: &Appearance, focus: Option<Choice>) -> Element<'static, Message> {
    let surfaces = appearance.surfaces();
    let variants = MATERIALS
        .into_iter()
        .zip(DETAILS)
        .map(|((id, name), detail)| {
            let mut sample = appearance.clone();
            sample.theme = id.into();
            let choice = material_choice(
                id,
                &sample,
                name,
                detail,
                appearance.theme == id,
                focus == Some(Choice::Material(id)),
            );
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
        let focused = focus == Some(Choice::Mode(id));
        button(container(text(label).size(12)).center_x(68))
            .padding([5, 0])
            .style(move |theme, status| {
                let style = action(theme, status);
                let style = button::Style {
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
                };
                focus_ring(style, theme, focused)
            })
            .on_press(Message::AppearanceMode(id))
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
        accent_group(appearance, &ACCENTS[..10], surfaces, focus),
        text("Glow").size(10).color(surfaces.muted),
        accent_group(appearance, &ACCENTS[10..], surfaces, focus),
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
                row![modes, accents].spacing(40).wrap().vertical_spacing(18),
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
    id: &'static str,
    appearance: &Appearance,
    name: &'static str,
    detail: &'static str,
    selected: bool,
    focused: bool,
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
        row![
            sample_line(palette.primary, iced::Length::FillPortion(65)),
            Space::new().width(iced::Length::FillPortion(35)),
        ],
        sample_line(surfaces.line, Fill),
        sample_line(surfaces.line, Fill),
    ]
    .spacing(8);
    let preview = container(column![
        crate::desktop::material::chrome(appearance, surfaces, 18),
        library::line(surfaces),
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
    let preview = container(preview)
        .padding(2)
        .width(Fill)
        .style(move |theme: &Theme| {
            if selected || focused {
                container::Style::default().border(
                    border::rounded(7)
                        .color(if focused {
                            theme.palette().text
                        } else {
                            theme.palette().primary
                        })
                        .width(2),
                )
            } else {
                container::Style::default()
            }
        });
    button(
        column![
            preview,
            text(name).size(12).font(bold()),
            text(detail).size(10).color(surfaces.muted),
        ]
        .spacing(6)
        .width(Fill),
    )
    .padding(0)
    .width(Fill)
    .style(move |theme, status| button::Style {
        background: None,
        ..action(theme, status)
    })
    .on_press(Message::Material(id))
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
    focus: Option<Choice>,
) -> Element<'static, Message> {
    row(choices.iter().map(|&(id, label)| {
        let mut sample = appearance.clone();
        sample.accent = id.into();
        let color = sample.theme().palette().primary;
        let selected = appearance.accent == id;
        let focused = focus == Some(Choice::Accent(id));
        let glow = id.starts_with("glow-");
        let dot = container(Space::new()).center(19).style(move |_| {
            container::Style::default()
                .background(color)
                .border(
                    border::rounded(10)
                        .width(if glow { 1 } else { 0 })
                        .color(color),
                )
                .shadow(Shadow {
                    color: color.scale_alpha(if glow { 0.5 } else { 0.0 }),
                    blur_radius: 8.0,
                    ..Shadow::default()
                })
        });
        tooltip(
            button(dot)
                .padding(3)
                .style(move |theme, status| {
                    focus_ring(
                        button::Style {
                            background: Some(surfaces.raised.into()),
                            border: border::rounded(15).color(color).width(if selected {
                                2
                            } else {
                                0
                            }),
                            ..action(theme, status)
                        },
                        theme,
                        focused,
                    )
                })
                .on_press(Message::Accent(id)),
            text(label).size(11),
            tooltip::Position::Top,
        )
        .into()
    }))
    .spacing(11)
    .align_y(Alignment::Center)
    .wrap()
    .vertical_spacing(10)
    .into()
}
