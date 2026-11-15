use iced::{
    Task,
    advanced::widget::{
        operate,
        operation::{focusable, scope},
    },
    widget::operation::{AbsoluteOffset, scroll_to},
};

use crate::appearance::{ACCENTS, MATERIALS, MODES};

use super::{Message, library_sources::Sources, sidebar::DIVIDER, source_removal::Launcher};

pub(super) const SCROLL: &str = "appearance-settings";
pub(super) const LIBRARY: &str = "library-window";

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum Choice {
    Material(&'static str),
    Mode(&'static str),
    Accent(&'static str),
    RemoveSource(Launcher),
    Divider,
}

impl Choice {
    pub(super) fn next(current: Option<&Self>, backwards: bool, sources: &Sources) -> Self {
        let remove_actions = sources
            .snapshot
            .as_ref()
            .filter(|_| !sources.busy())
            .into_iter()
            .flat_map(|snapshot| &snapshot.libraries)
            .map(|source| Self::RemoveSource(Launcher::Settings(source.id.clone())));
        let mut choices = MATERIALS
            .into_iter()
            .map(|(id, _)| Self::Material(id))
            .chain(MODES.into_iter().map(|(id, _)| Self::Mode(id)))
            .chain(ACCENTS.into_iter().map(|(id, _)| Self::Accent(id)))
            .chain(remove_actions)
            .chain([Self::Divider])
            .collect::<Vec<_>>();
        let length = choices.len();
        let index = match (
            current.and_then(|item| choices.iter().position(|choice| choice == item)),
            backwards,
        ) {
            (Some(index), true) => (index + length - 1) % length,
            (Some(index), false) => (index + 1) % length,
            (None, true) => length - 1,
            (None, false) => 0,
        };
        choices.swap_remove(index)
    }

    pub(super) fn message(&self) -> Option<Message> {
        Some(match self {
            Self::Material(id) => Message::Material(id),
            Self::Mode(id) => Message::AppearanceMode(id),
            Self::Accent(id) => Message::Accent(id),
            Self::RemoveSource(_) | Self::Divider => return None,
        })
    }

    pub(super) fn take_focus(&self) -> Task<Message> {
        let y = match self {
            Self::Material(id) => {
                let index = MATERIALS
                    .iter()
                    .position(|(material, _)| material == id)
                    .unwrap_or_default();
                (index / 3) as f32 * 125.0
            }
            Self::Mode(_) => 380.0,
            Self::Accent(_) => 440.0,
            Self::RemoveSource(launcher) => return launcher.focus(),
            Self::Divider => {
                return operate(scope(
                    LIBRARY.into(),
                    focusable::focus::<()>(DIVIDER.into()),
                ))
                .discard();
            }
        };
        Task::batch([
            operate(scope(LIBRARY.into(), focusable::unfocus::<()>())).discard(),
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
