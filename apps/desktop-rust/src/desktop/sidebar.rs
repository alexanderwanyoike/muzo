use super::windows::LIBRARY_SIZE;
use crate::appearance::sidebar::SidebarWidth;

pub(super) const SPLITTER_WIDTH: f32 = 1.0;

#[derive(Debug, Clone, Copy)]
pub enum Resize {
    Preview(f32),
    Commit,
    Cancel,
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
            Resize::Preview(requested) => {
                let width = SidebarWidth::new(requested);
                let available = self.viewport - SPLITTER_WIDTH;
                self.preview = (width.visible(available) != self.preferred.visible(available))
                    .then_some(width);
                None
            }
            Resize::Commit => {
                let preview = self.preview.take()?;
                let width = SidebarWidth::new(preview.visible(self.viewport - SPLITTER_WIDTH));
                self.set(width);
                Some(width)
            }
            Resize::Cancel => {
                self.preview = None;
                None
            }
        }
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
