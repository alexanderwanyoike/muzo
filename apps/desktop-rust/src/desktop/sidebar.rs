use iced::widget::pane_grid::{Axis, Split, State};

use super::windows::LIBRARY_SIZE;
use crate::appearance::sidebar::SidebarWidth;

pub(super) const LIBRARY_PADDING: f32 = 20.0;
pub(super) const SPLITTER_SPACING: f32 = 12.0;

#[derive(Clone, Copy)]
pub(super) enum Pane {
    Sidebar,
    Content,
}

pub(super) struct Sidebar {
    pub grid: State<Pane>,
    split: Split,
    viewport: f32,
    width: SidebarWidth,
}

impl Default for Sidebar {
    fn default() -> Self {
        let (mut grid, sidebar) = State::new(Pane::Sidebar);
        let (_, split) = grid
            .split(Axis::Vertical, sidebar, Pane::Content)
            .expect("a fresh pane grid has one pane to split");
        let mut layout = Self {
            grid,
            split,
            viewport: LIBRARY_SIZE.width,
            width: SidebarWidth::default(),
        };
        layout.update_grid();
        layout
    }
}

impl Sidebar {
    pub(super) fn layout_width(&mut self, viewport: f32) {
        self.viewport = viewport;
        self.update_grid();
    }

    pub(super) fn set(&mut self, width: SidebarWidth) {
        self.width = width;
        self.update_grid();
    }

    pub(super) fn drag(&mut self, ratio: f32) -> SidebarWidth {
        let requested = ratio * self.content_width() - SPLITTER_SPACING / 2.0;
        let desired = SidebarWidth::new(requested);
        let clamped = SidebarWidth::new(desired.visible(self.pane_space()));
        self.set(clamped);
        clamped
    }

    pub(super) fn visible_width(&self) -> f32 {
        self.width.visible(self.pane_space())
    }

    pub(super) fn preferred_width(&self) -> SidebarWidth {
        self.width
    }

    fn pane_space(&self) -> f32 {
        self.content_width() - SPLITTER_SPACING
    }

    fn content_width(&self) -> f32 {
        (self.viewport - LIBRARY_PADDING * 2.0).max(1.0)
    }

    fn update_grid(&mut self) {
        let ratio = (self.visible_width() + SPLITTER_SPACING / 2.0) / self.content_width();
        self.grid.resize(self.split, ratio.clamp(0.0, 1.0));
    }
}
