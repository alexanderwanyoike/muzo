use iced::{
    Task,
    advanced::widget::{operate, operation::focusable::unfocus},
    widget::operation::{AbsoluteOffset, scroll_to},
};

use crate::appearance::{ACCENTS, MATERIALS, MODES};

use super::Message;

pub(super) const SCROLL: &str = "appearance-settings";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Choice {
    Material(&'static str),
    Mode(&'static str),
    Accent(&'static str),
}

impl Choice {
    pub(super) fn next(current: Option<Self>, backwards: bool) -> Self {
        let choices = MATERIALS
            .into_iter()
            .map(|(id, _)| Self::Material(id))
            .chain(MODES.into_iter().map(|(id, _)| Self::Mode(id)))
            .chain(ACCENTS.into_iter().map(|(id, _)| Self::Accent(id)))
            .collect::<Vec<_>>();
        let length = choices.len();
        let index = match (
            current.and_then(|item| choices.iter().position(|choice| *choice == item)),
            backwards,
        ) {
            (Some(index), true) => (index + length - 1) % length,
            (Some(index), false) => (index + 1) % length,
            (None, true) => length - 1,
            (None, false) => 0,
        };
        choices[index]
    }

    pub(super) fn message(self) -> Message {
        match self {
            Self::Material(id) => Message::Material(id),
            Self::Mode(id) => Message::AppearanceMode(id),
            Self::Accent(id) => Message::Accent(id),
        }
    }

    pub(super) fn take_focus(self) -> Task<Message> {
        let y = match self {
            Self::Material(id) => {
                let index = MATERIALS
                    .iter()
                    .position(|(material, _)| *material == id)
                    .unwrap_or_default();
                (index / 3) as f32 * 125.0
            }
            Self::Mode(_) => 380.0,
            Self::Accent(_) => 440.0,
        };
        Task::batch([
            operate(unfocus::<()>()).discard(),
            scroll_to(
                SCROLL,
                AbsoluteOffset {
                    x: None,
                    y: Some(y),
                },
            ),
        ])
    }
}
