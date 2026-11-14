use std::collections::VecDeque;

use iced::{
    Rectangle, Task, Vector,
    advanced::widget::{
        Id, Operation, operate,
        operation::{
            Outcome,
            focusable::Focusable,
            scope,
            scrollable::{AbsoluteOffset, Scrollable},
        },
    },
};

use super::{Message, appearance_focus::LIBRARY};

pub(super) const SONGS_SCROLL: &str = "indexed-songs";
const FOCUS_MARGIN: f32 = 4.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Field {
    Title,
    Artist,
    Album,
    Track,
    Disc,
    Genre,
    Year,
}

impl Field {
    const ORDER: [Self; 7] = [
        Self::Title,
        Self::Artist,
        Self::Album,
        Self::Track,
        Self::Disc,
        Self::Genre,
        Self::Year,
    ];

    pub(super) const fn index(self) -> usize {
        self as usize
    }

    pub(super) const fn label(self) -> &'static str {
        match self {
            Self::Title => "Title",
            Self::Artist => "Artist",
            Self::Album => "Album",
            Self::Track => "Track",
            Self::Disc => "Disc",
            Self::Genre => "Genre",
            Self::Year => "Year",
        }
    }

    pub(super) const fn id(self) -> &'static str {
        match self {
            Self::Title => "metadata-title",
            Self::Artist => "metadata-artist",
            Self::Album => "metadata-album",
            Self::Track => "metadata-track",
            Self::Disc => "metadata-disc",
            Self::Genre => "metadata-genre",
            Self::Year => "metadata-year",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Control {
    Input(Field),
    Save,
    Clear,
    Cancel,
}

impl Control {
    pub(super) fn next(
        current: Option<Self>,
        backwards: bool,
        can_save: bool,
        can_clear: bool,
    ) -> Self {
        let mut order: Vec<_> = Field::ORDER.into_iter().map(Self::Input).collect();
        order.extend(can_save.then_some(Self::Save));
        order.extend(can_clear.then_some(Self::Clear));
        order.push(Self::Cancel);
        if backwards {
            order.reverse();
        }
        match current.and_then(|current| order.iter().position(|item| *item == current)) {
            Some(index) => order[(index + 1) % order.len()],
            None => order[0],
        }
    }

    pub(super) fn message(self) -> Option<Message> {
        match self {
            Self::Input(_) => None,
            Self::Save => Some(Message::SaveTrackEdit(false)),
            Self::Clear => Some(Message::SaveTrackEdit(true)),
            Self::Cancel => Some(Message::CloseTrackEdit),
        }
    }

    pub(super) const fn id(self) -> &'static str {
        match self {
            Self::Input(field) => field.id(),
            Self::Save => "metadata-save",
            Self::Clear => "metadata-clear",
            Self::Cancel => "metadata-cancel",
        }
    }

    pub(super) fn focus(self, serial: u64) -> Task<Message> {
        let change = FocusControl {
            control: self,
            serial,
            reveal: reveal(self),
        };
        operate(scope(LIBRARY.into(), change)).map(Message::TrackEditorFocusApplied)
    }
}

#[derive(Default)]
pub(super) struct Navigation {
    serial: u64,
    active: Option<u64>,
    queued: VecDeque<KeyboardAction>,
}

enum KeyboardAction {
    Tab(bool),
    Activate,
}

impl Navigation {
    pub(super) fn request(&mut self, backwards: bool) -> Task<Message> {
        if self.active.is_some() {
            self.queued.push_back(KeyboardAction::Tab(backwards));
            return Task::none();
        }
        self.serial += 1;
        let serial = self.serial;
        self.active = Some(serial);
        operate(scope(LIBRARY.into(), FocusedField::default()))
            .map(move |field| Message::TrackEditorTab(serial, backwards, field))
    }

    pub(super) fn is_active(&self, serial: u64) -> bool {
        self.active == Some(serial)
    }

    pub(super) fn finish(&mut self, serial: u64) -> Task<Message> {
        if !self.is_active(serial) {
            return Task::none();
        }
        self.active = None;
        self.advance()
    }

    pub(super) fn defer_activation(&mut self) -> bool {
        if self.active.is_none() && self.queued.is_empty() {
            return false;
        }
        self.queued.push_back(KeyboardAction::Activate);
        true
    }

    pub(super) fn focus_pending(&self) -> bool {
        self.active.is_some()
    }

    pub(super) fn advance(&mut self) -> Task<Message> {
        match self.queued.pop_front() {
            Some(KeyboardAction::Tab(backwards)) => self.request(backwards),
            Some(KeyboardAction::Activate) => Task::done(Message::ActivateTrackEditor),
            None => Task::none(),
        }
    }

    pub(super) fn reset(&mut self) {
        self.active = None;
        self.queued.clear();
    }
}

#[derive(Default)]
struct FocusedField(Option<Field>);

impl Operation<Option<Field>> for FocusedField {
    fn traverse(&mut self, visit: &mut dyn FnMut(&mut dyn Operation<Option<Field>>)) {
        visit(self);
    }

    fn focusable(&mut self, id: Option<&Id>, _: Rectangle, state: &mut dyn Focusable) {
        if state.is_focused() {
            self.0 = Field::ORDER
                .into_iter()
                .find(|field| id == Some(&field.id().into()));
        }
    }

    fn finish(&self) -> Outcome<Option<Field>> {
        Outcome::Some(self.0)
    }
}

struct FocusControl {
    control: Control,
    serial: u64,
    reveal: RevealControl,
}

impl Operation<u64> for FocusControl {
    fn traverse(&mut self, visit: &mut dyn FnMut(&mut dyn Operation<u64>)) {
        visit(self);
    }

    fn container(&mut self, id: Option<&Id>, bounds: Rectangle) {
        self.reveal.container(id, bounds);
    }

    fn focusable(&mut self, id: Option<&Id>, bounds: Rectangle, state: &mut dyn Focusable) {
        self.reveal.focusable(id, bounds, state);
        if matches!(self.control, Control::Input(_)) && id == Some(&self.reveal.target) {
            state.focus();
        } else {
            state.unfocus();
        }
    }

    fn scrollable(
        &mut self,
        id: Option<&Id>,
        bounds: Rectangle,
        content: Rectangle,
        offset: Vector,
        state: &mut dyn Scrollable,
    ) {
        self.reveal.scrollable(id, bounds, content, offset, state);
    }

    fn finish(&self) -> Outcome<u64> {
        let Some(offset) = self.reveal.offset() else {
            return Outcome::Some(self.serial);
        };
        Outcome::Chain(Box::new(ApplyScroll {
            offset,
            serial: self.serial,
        }))
    }
}

struct ApplyScroll {
    offset: AbsoluteOffset<Option<f32>>,
    serial: u64,
}

impl Operation<u64> for ApplyScroll {
    fn traverse(&mut self, visit: &mut dyn FnMut(&mut dyn Operation<u64>)) {
        visit(self);
    }

    fn scrollable(
        &mut self,
        id: Option<&Id>,
        _: Rectangle,
        _: Rectangle,
        _: Vector,
        state: &mut dyn Scrollable,
    ) {
        if id == Some(&SONGS_SCROLL.into()) {
            state.scroll_to(self.offset);
        }
    }

    fn finish(&self) -> Outcome<u64> {
        Outcome::Some(self.serial)
    }
}

pub(super) fn reveal(control: Control) -> RevealControl {
    reveal_songs_control(control.id())
}

pub(super) fn reveal_songs_control(id: &'static str) -> RevealControl {
    RevealControl {
        target: id.into(),
        viewport: None,
        bounds: None,
    }
}

pub(super) struct RevealControl {
    target: Id,
    viewport: Option<(Rectangle, f32)>,
    bounds: Option<Rectangle>,
}

impl Operation for RevealControl {
    fn traverse(&mut self, visit: &mut dyn FnMut(&mut dyn Operation)) {
        visit(self);
    }

    fn container(&mut self, id: Option<&Id>, bounds: Rectangle) {
        if id == Some(&self.target) {
            self.bounds = Some(bounds);
        }
    }

    fn focusable(&mut self, id: Option<&Id>, bounds: Rectangle, _: &mut dyn Focusable) {
        self.container(id, bounds);
    }

    fn scrollable(
        &mut self,
        id: Option<&Id>,
        bounds: Rectangle,
        _: Rectangle,
        translation: Vector,
        _: &mut dyn Scrollable,
    ) {
        if id == Some(&SONGS_SCROLL.into()) {
            self.viewport = Some((bounds, translation.y));
        }
    }

    fn finish(&self) -> Outcome<()> {
        self.offset().map_or(Outcome::None, |offset| {
            Outcome::Chain(Box::new(
                iced::advanced::widget::operation::scrollable::scroll_to(
                    SONGS_SCROLL.into(),
                    offset,
                ),
            ))
        })
    }
}

impl RevealControl {
    fn offset(&self) -> Option<AbsoluteOffset<Option<f32>>> {
        let (target, (viewport, offset)) = self.bounds.zip(self.viewport)?;
        let top = target.y - FOCUS_MARGIN;
        let bottom = target.y + target.height + FOCUS_MARGIN;
        let y = if top < viewport.y + offset {
            top - viewport.y
        } else if bottom > viewport.y + viewport.height + offset {
            bottom - viewport.y - viewport.height
        } else {
            return None;
        };
        Some(AbsoluteOffset {
            x: None,
            y: Some(y.max(0.0)),
        })
    }
}
