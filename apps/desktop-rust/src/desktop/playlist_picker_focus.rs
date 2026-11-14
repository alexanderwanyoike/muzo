use iced::{
    Task,
    advanced::widget::{
        operate,
        operation::{focusable::unfocus, scope},
    },
};

use super::{Message, appearance_focus::LIBRARY, track_editor_focus::reveal_songs_control};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Control {
    Choice,
    Add,
    Cancel,
}

impl Control {
    pub(super) fn next(
        current: Option<Self>,
        backwards: bool,
        has_choice: bool,
        can_add: bool,
    ) -> Self {
        let order = [
            has_choice.then_some(Self::Choice),
            (has_choice && can_add).then_some(Self::Add),
            Some(Self::Cancel),
        ];
        let mut controls = order.into_iter().flatten();
        let first = if backwards {
            controls.next_back()
        } else {
            controls.next()
        }
        .unwrap_or(Self::Cancel);
        let mut previous = first;
        while let Some(next) = if backwards {
            controls.next_back()
        } else {
            controls.next()
        } {
            if current == Some(previous) {
                return next;
            }
            previous = next;
        }
        first
    }

    pub(super) const fn id(self) -> &'static str {
        match self {
            Self::Choice => "track-playlist-choice",
            Self::Add => "track-playlist-add",
            Self::Cancel => "track-playlist-cancel",
        }
    }

    pub(super) fn message(self) -> Option<Message> {
        match self {
            Self::Choice => None,
            Self::Add => Some(Message::SubmitPlaylistPicker),
            Self::Cancel => Some(Message::ClosePlaylistPicker),
        }
    }

    pub(super) fn reveal(self) -> Task<Message> {
        operate(scope(LIBRARY.into(), unfocus::<()>()))
            .discard()
            .chain(operate(reveal_songs_control(self.id())).discard())
    }
}
