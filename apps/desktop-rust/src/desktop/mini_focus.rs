use super::{Message, Session};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Control {
    Library,
    Queue,
    Previous,
    Playback,
    Next,
    Volume,
    Retry,
    Track(usize),
}

impl Control {
    const ORDER: [Self; 7] = [
        Self::Library,
        Self::Queue,
        Self::Previous,
        Self::Playback,
        Self::Next,
        Self::Volume,
        Self::Retry,
    ];

    pub(super) fn next(
        current: Option<Self>,
        backwards: bool,
        tracks: usize,
        enabled: impl Fn(Self) -> bool,
    ) -> Option<Self> {
        let mut controls = Self::ORDER
            .into_iter()
            .chain((0..tracks).map(Self::Track))
            .filter(|control| enabled(*control));
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

    pub(super) fn enabled(
        self,
        session: Option<&Session>,
        retry: bool,
        queue_pending: bool,
        queue_open: bool,
    ) -> bool {
        self.message(session, retry, queue_pending, queue_open)
            .is_some()
            || (self == Self::Volume && session.is_some_and(|session| session.current().is_some()))
    }

    pub(super) fn message(
        self,
        session: Option<&Session>,
        retry: bool,
        queue_pending: bool,
        queue_open: bool,
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
            Self::Volume => None,
            Self::Retry => retry.then_some(Message::RetryTrack),
            Self::Track(track) => (queue_open && session.is_some_and(|s| track < s.tracks().len()))
                .then_some(if index == Some(track) {
                    Message::TogglePlayback
                } else {
                    Message::Select(track)
                }),
        }
    }
}
