use iced::{
    Alignment, Color, Element, Fill, border,
    widget::{Space, button, container, mouse_area, row, svg, text, tooltip},
    window,
};

use crate::appearance::Appearance;

use super::{Icon, Message, bold, icon_tinted, panel};

const TITLE_HEIGHT: u32 = 32;
const CAPTION_WIDTH: u32 = 32;
const CAPTION_GLYPH: u32 = 10;
pub(super) const PANE_GUTTER: f32 = 8.0;
pub(super) const PANE_RADIUS: f32 = 12.0;

pub(super) fn pane_style(fill: Color) -> container::Style {
    container::Style::default()
        .background(fill)
        .border(border::rounded(PANE_RADIUS))
}

#[cfg(not(target_os = "macos"))]
pub(in crate::desktop) fn resize_edges(
    id: window::Id,
    content: Element<'_, Message>,
) -> Element<'_, Message> {
    use iced::{
        Length::Fixed,
        alignment::{Horizontal, Vertical},
        mouse::Interaction,
        widget::stack,
        window::Direction,
    };

    const EDGE: iced::Length = Fixed(5.0);
    let grip = |direction, horizontal, vertical, width, height, cursor| {
        container(
            mouse_area(Space::new().width(width).height(height))
                .on_press(Message::ResizeWindow(id, direction))
                .interaction(cursor),
        )
        .align_x(horizontal)
        .align_y(vertical)
        .width(Fill)
        .height(Fill)
    };
    use Horizontal::{Center as Middle, Left, Right};
    use Interaction::{
        ResizingDiagonallyDown, ResizingDiagonallyUp, ResizingHorizontally, ResizingVertically,
    };
    use Vertical::{Bottom, Center, Top};
    let fill = iced::Length::Fill;
    stack![
        content,
        grip(
            Direction::West,
            Left,
            Center,
            EDGE,
            fill,
            ResizingHorizontally
        ),
        grip(
            Direction::East,
            Right,
            Center,
            EDGE,
            fill,
            ResizingHorizontally
        ),
        grip(
            Direction::North,
            Middle,
            Top,
            fill,
            EDGE,
            ResizingVertically
        ),
        grip(
            Direction::South,
            Middle,
            Bottom,
            fill,
            EDGE,
            ResizingVertically
        ),
        grip(
            Direction::NorthWest,
            Left,
            Top,
            EDGE,
            EDGE,
            ResizingDiagonallyDown
        ),
        grip(
            Direction::NorthEast,
            Right,
            Top,
            EDGE,
            EDGE,
            ResizingDiagonallyUp
        ),
        grip(
            Direction::SouthWest,
            Left,
            Bottom,
            EDGE,
            EDGE,
            ResizingDiagonallyUp
        ),
        grip(
            Direction::SouthEast,
            Right,
            Bottom,
            EDGE,
            EDGE,
            ResizingDiagonallyDown
        ),
    ]
    .into()
}

pub(in crate::desktop) fn title_bar<'a>(
    id: window::Id,
    appearance: &Appearance,
    tools: Element<'a, Message>,
) -> Element<'a, Message> {
    let background = appearance.surfaces().chrome;
    let accent = appearance.accent_ink();
    let brand = row![
        icon_tinted(Icon::Music, move |_| accent)
            .width(16)
            .height(16),
        text("Muzo").font(bold()).size(13),
        Space::new().width(Fill),
    ]
    .spacing(8)
    .align_y(Alignment::Center);
    let drag = mouse_area(
        container(brand)
            .padding([0, 12])
            .center_y(TITLE_HEIGHT)
            .width(Fill),
    )
    .on_press(Message::DragWindow(id))
    .on_double_click(Message::ToggleWindowMaximized(id));
    container(
        row![
            drag,
            tools,
            caption(Caption::Minimize, "Minimize", Message::MinimizeWindow(id)),
            caption(
                Caption::Maximize,
                "Maximize or restore",
                Message::ToggleWindowMaximized(id)
            ),
            caption(Caption::Close, "Close", Message::CloseRequested(id)),
        ]
        .align_y(Alignment::Center),
    )
    .width(Fill)
    .style(move |_| container::Style::default().background(background))
    .into()
}

#[derive(Clone, Copy)]
enum Caption {
    Minimize,
    Maximize,
    Close,
}

impl Caption {
    fn glyph(self) -> svg::Svg<'static> {
        let path = match self {
            Self::Minimize => "M1.5 8.5h7",
            Self::Maximize => "M1.5 1.5h7v7h-7z",
            Self::Close => "m1.5 2 7 6m0-6-7 6",
        };
        let markup = format!(
            "<svg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 10 10' fill='none' stroke='black' stroke-width='1.4' stroke-linecap='round'><path d='{path}'/></svg>"
        );
        svg(svg::Handle::from_memory(markup.into_bytes()))
            .width(CAPTION_GLYPH)
            .height(CAPTION_GLYPH)
            .style(|theme: &iced::Theme, _| svg::Style {
                color: Some(theme.palette().text),
            })
    }
}

fn caption(kind: Caption, name: &'static str, message: Message) -> Element<'static, Message> {
    tooltip(
        button(container(kind.glyph()).center(Fill))
            .padding(0)
            .width(CAPTION_WIDTH)
            .height(TITLE_HEIGHT)
            .on_press(message)
            .style(move |theme, state| {
                let active = matches!(state, button::Status::Hovered | button::Status::Pressed);
                let fill = if matches!(kind, Caption::Close) {
                    iced::color!(0xb44542).scale_alpha(0.82)
                } else {
                    theme.palette().text.scale_alpha(0.08)
                };
                button::Style {
                    background: active.then(|| fill.into()),
                    border: border::rounded(0),
                    ..button::Style::default()
                }
            }),
        text(name).size(11),
        tooltip::Position::Bottom,
    )
    .style(panel)
    .into()
}
