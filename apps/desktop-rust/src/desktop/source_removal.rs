use crate::catalogue::Library;

pub(super) struct Removal {
    pub source: Library,
    pub busy: bool,
    pub error: Option<String>,
    pub cancel_focused: bool,
    pub show_keyboard_focus: bool,
    space_armed: bool,
}

impl Removal {
    pub fn new(source: Library) -> Self {
        Self {
            source,
            busy: false,
            error: None,
            cancel_focused: true,
            show_keyboard_focus: false,
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
