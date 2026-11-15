use crate::catalogue::Library;
use iced::{
    Task,
    advanced::widget::{
        Id, operate,
        operation::{focusable, scope},
    },
    widget::operation::{AbsoluteOffset, RelativeOffset, scroll_by, snap_to},
};

use super::Message;

pub(super) const DETAILS: &str = "source-removal-details";
pub(super) const SIDEBAR: &str = "sources-sidebar-list";

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Launcher {
    Sidebar(String),
    Settings(String),
}

impl Launcher {
    pub(super) fn widget_id(&self) -> Id {
        match self {
            Self::Sidebar(source) => format!("source-remove-sidebar:{source}"),
            Self::Settings(source) => format!("source-remove-settings:{source}"),
        }
        .into()
    }

    pub(super) fn source_id(&self) -> &str {
        match self {
            Self::Sidebar(source) | Self::Settings(source) => source,
        }
    }

    pub(super) fn reveal(&self) -> Task<Message> {
        let scroll = match self {
            Self::Sidebar(_) => SIDEBAR,
            Self::Settings(_) => super::appearance_focus::SCROLL,
        };
        operate(super::track_editor_focus::reveal_scroll_item(
            scroll,
            self.widget_id(),
        ))
        .discard()
    }

    pub(super) fn focus(&self) -> Task<Message> {
        operate(scope(
            super::appearance_focus::LIBRARY.into(),
            focusable::focus::<()>(self.widget_id()),
        ))
        .discard()
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Scroll {
    By(f32),
    Start,
    End,
}

impl Scroll {
    pub(super) fn apply(self) -> Task<Message> {
        match self {
            Self::By(y) => scroll_by(DETAILS, AbsoluteOffset { x: 0.0, y }),
            Self::Start | Self::End => snap_to(
                DETAILS,
                RelativeOffset {
                    x: None,
                    y: Some(if matches!(self, Self::End) { 1.0 } else { 0.0 }),
                },
            ),
        }
    }
}

pub(super) struct Removal {
    pub source: Library,
    pub busy: bool,
    pub error: Option<String>,
    pub cancel_focused: bool,
    pub show_keyboard_focus: bool,
    pub launcher: Option<Launcher>,
    space_armed: bool,
}

impl Removal {
    pub fn new(source: Library, launcher: Option<Launcher>) -> Self {
        Self {
            source,
            busy: false,
            error: None,
            cancel_focused: true,
            show_keyboard_focus: launcher.is_some(),
            launcher,
            space_armed: false,
        }
    }

    pub fn start(&mut self) {
        self.busy = true;
        self.error = None;
        self.space_armed = false;
    }

    pub fn failed(&mut self, error: String) {
        self.busy = false;
        self.error = Some(error);
        self.cancel_focused = true;
    }

    pub fn focus_next(&mut self) {
        if !self.busy {
            self.cancel_focused = !self.cancel_focused;
            self.show_keyboard_focus = true;
            self.space_armed = false;
        }
    }

    pub fn set_space_armed(&mut self, armed: bool) {
        self.space_armed = armed && !self.busy;
    }

    pub fn take_space_armed(&mut self) -> bool {
        std::mem::take(&mut self.space_armed)
    }
}
