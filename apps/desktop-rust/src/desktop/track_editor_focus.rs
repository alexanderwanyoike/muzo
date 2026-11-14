use iced::{
    Rectangle, Task, Vector,
    advanced::widget::{
        Id, Operation, operate,
        operation::{
            Outcome,
            focusable::{Focusable, focus, unfocus},
            scope,
            scrollable::{AbsoluteOffset, Scrollable, scroll_to},
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

    pub(super) fn focus(self) -> Task<Message> {
        let focus = if let Self::Input(field) = self {
            operate(scope(LIBRARY.into(), focus::<()>(field.id().into()))).discard()
        } else {
            operate(scope(LIBRARY.into(), unfocus::<()>())).discard()
        };
        focus.chain(operate(reveal(self)).discard())
    }
}

pub(super) fn observe(track: String, backwards: bool) -> Task<Message> {
    operate(scope(LIBRARY.into(), FocusedField::default()))
        .map(move |field| Message::TrackEditorTab(track.clone(), backwards, field))
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

pub(super) fn reveal(control: Control) -> impl Operation {
    RevealControl {
        target: control.id().into(),
        viewport: None,
        bounds: None,
    }
}

struct RevealControl {
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
        let Some((target, (viewport, offset))) = self.bounds.zip(self.viewport) else {
            return Outcome::None;
        };
        let top = target.y - FOCUS_MARGIN;
        let bottom = target.y + target.height + FOCUS_MARGIN;
        let y = if top < viewport.y + offset {
            top - viewport.y
        } else if bottom > viewport.y + viewport.height + offset {
            bottom - viewport.y - viewport.height
        } else {
            return Outcome::None;
        };
        Outcome::Chain(Box::new(scroll_to(
            SONGS_SCROLL.into(),
            AbsoluteOffset {
                x: None,
                y: Some(y.max(0.0)),
            },
        )))
    }
}
