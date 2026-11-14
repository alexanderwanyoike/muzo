use super::{Message, Session};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Control {
    Library,
    Queue,
    Previous,
    Playback,
    Next,
    Retry,
}

impl Control {
    const ORDER: [Self; 6] = [
        Self::Library,
        Self::Queue,
        Self::Previous,
        Self::Playback,
        Self::Next,
        Self::Retry,
    ];

    pub(super) fn next(
        current: Option<Self>,
        backwards: bool,
        enabled: impl Fn(Self) -> bool,
    ) -> Option<Self> {
        let mut controls = Self::ORDER.into_iter().filter(|control| enabled(*control));
        let mut take = || {
            if backwards {
                controls.next_back()
            } else {
                controls.next()
            }
        };
        let first = take();
        let Some(current) = current else {
            return first;
        };
        let mut next = first;
        while let Some(control) = next {
            next = take();
            if control == current {
                return next.or(first);
            }
        }
        first
    }

    pub(super) fn message(
        self,
        session: Option<&Session>,
        retry: bool,
        queue_pending: bool,
    ) -> Option<Message> {
        let index = session.and_then(Session::current_index);
        match self {
            Self::Library => Some(Message::ShowLibrary),
            Self::Queue => (!queue_pending).then_some(Message::ToggleQueue),
            Self::Previous => index
                .is_some_and(|index| index > 0)
                .then_some(Message::Previous),
            Self::Playback => (retry || index.is_some()).then_some(Message::TogglePlayback),
            Self::Next => session
                .zip(index)
                .is_some_and(|(session, index)| index + 1 < session.tracks().len())
                .then_some(Message::Next),
            Self::Retry => retry.then_some(Message::RetryTrack),
        }
    }
}
