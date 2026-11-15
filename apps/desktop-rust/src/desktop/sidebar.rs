use super::windows::LIBRARY_SIZE;
use crate::appearance::sidebar::SidebarWidth;

pub(super) const SPLITTER_WIDTH: f32 = 1.0;
const KEYBOARD_STEP: f32 = 8.0;

#[derive(Debug, Clone, Copy)]
pub enum Adjustment {
    Narrower,
    Wider,
    Minimum,
    Maximum,
}

#[derive(Debug, Clone, Copy)]
pub enum Resize {
    Viewport(f32),
    Preview(f32),
    Commit(f32),
    Cancel,
    Reset,
    Adjust(Adjustment),
}

pub(super) struct Sidebar {
    viewport: f32,
    preferred: SidebarWidth,
    preview: Option<SidebarWidth>,
}

impl Default for Sidebar {
    fn default() -> Self {
        Self {
            viewport: LIBRARY_SIZE.width,
            preferred: SidebarWidth::default(),
            preview: None,
        }
    }
}

impl Sidebar {
    pub(super) fn layout_width(&mut self, viewport: f32) {
        self.viewport = viewport;
    }

    pub(super) fn set(&mut self, width: SidebarWidth) {
        self.preferred = width;
    }

    pub(super) fn resize(&mut self, action: Resize) -> Option<SidebarWidth> {
        match action {
            Resize::Viewport(width) => {
                self.layout_width(width);
                None
            }
            Resize::Preview(requested) => {
                self.preview = Some(SidebarWidth::new(requested));
                None
            }
            Resize::Commit(requested) => {
                self.preview = None;
                Some(self.commit(SidebarWidth::new(requested)))
            }
            Resize::Cancel => {
                self.preview = None;
                None
            }
            Resize::Reset => {
                self.preview = None;
                Some(self.commit(SidebarWidth::default()))
            }
            Resize::Adjust(direction) => {
                let target = match direction {
                    Adjustment::Narrower => SidebarWidth::new(self.visible_width() - KEYBOARD_STEP),
                    Adjustment::Wider => SidebarWidth::new(self.visible_width() + KEYBOARD_STEP),
                    Adjustment::Minimum => SidebarWidth::MINIMUM,
                    Adjustment::Maximum => SidebarWidth::MAXIMUM,
                };
                self.preview = None;
                Some(self.commit(target))
            }
        }
    }

    fn commit(&mut self, target: SidebarWidth) -> SidebarWidth {
        let width = SidebarWidth::new(target.visible(self.viewport - SPLITTER_WIDTH));
        self.set(width);
        width
    }

    pub(super) fn visible_width(&self) -> f32 {
        self.preview
            .unwrap_or(self.preferred)
            .visible(self.viewport - SPLITTER_WIDTH)
    }

    #[cfg(test)]
    pub(super) fn preferred_width(&self) -> SidebarWidth {
        self.preferred
    }
}
