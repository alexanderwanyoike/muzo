use super::{Message, Operation};
use crate::catalogue::Library;

pub(super) struct Removal {
    pub source: Library,
    pub busy: bool,
    pub error: Option<String>,
    pub cancel_focused: bool,
}

impl Removal {
    pub fn new(source: Library) -> Self {
        Self {
            source,
            busy: false,
            error: None,
            cancel_focused: true,
        }
    }

    pub fn start(&mut self) {
        self.busy = true;
        self.error = None;
    }

    pub fn failed(&mut self, error: String) {
        self.busy = false;
        self.error = Some(error);
        self.cancel_focused = true;
    }

    pub fn focus_next(&mut self) {
        if !self.busy {
            self.cancel_focused = !self.cancel_focused;
        }
    }

    pub fn confirm(&self) -> Message {
        Message::SourceOperation(Operation::Remove(self.source.id.clone()))
    }
}
