use iced::{
    Alignment, Element, Fill, border,
    widget::{Space, button, container, mouse_area, row, text, tooltip},
    window,
};

use crate::appearance::Appearance;

use super::{Icon, Message, bold, icon_tinted, panel};

pub(in crate::desktop) const LIBRARY_TITLE_HEIGHT: u32 = 40;
pub(super) const MINI_TITLE_HEIGHT: u32 = 50;
const CAPTION_WIDTH: u32 = 32;

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
    height: u32,
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
            .center_y(height)
            .width(Fill),
    )
    .on_press(Message::DragWindow(id))
    .on_double_click(Message::ToggleWindowMaximized(id));
    container(
        row![
            drag,
            tools,
            caption(
                Icon::Minimize,
                "Minimize",
                Message::MinimizeWindow(id),
                height
            ),
            caption(
                Icon::Maximize,
                "Maximize or restore",
                Message::ToggleWindowMaximized(id),
                height
            ),
            caption(Icon::Close, "Close", Message::CloseRequested(id), height),
        ]
        .align_y(Alignment::Center),
    )
    .width(Fill)
    .style(move |_| container::Style::default().background(background))
    .into()
}

fn caption(
    icon: Icon,
    name: &'static str,
    message: Message,
    height: u32,
) -> Element<'static, Message> {
    tooltip(
        button(
            icon_tinted(icon, |theme| theme.palette().text)
                .width(16)
                .height(16),
        )
        .padding(0)
        .width(CAPTION_WIDTH)
        .height(height)
        .on_press(message)
        .style(move |theme, state| {
            let active = matches!(state, button::Status::Hovered | button::Status::Pressed);
            let fill = if matches!(icon, Icon::Close) {
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
