use std::{collections::HashMap, path::Path, time::Duration};

use iced::{
    Font, Pixels, Rectangle, Renderer, Size,
    advanced::{
        Layout, layout,
        widget::{Id, Operation, Tree},
    },
    window,
};

use crate::session::{AudioError, AudioPlayback, Session};

use super::{Desktop, Message};

struct LongRecording;

impl AudioPlayback for LongRecording {
    fn play(&mut self, _: &Path) -> Result<(), AudioError> {
        Ok(())
    }

    fn reset(&mut self) {}

    fn is_finished(&self) -> bool {
        false
    }

    fn volume(&self) -> f32 {
        1.0
    }

    fn set_volume(&mut self, _: f32) {}

    fn duration(&self) -> Duration {
        Duration::from_secs(39_605)
    }

    fn position(&self) -> Duration {
        Duration::from_secs(36_001)
    }

    fn is_paused(&self) -> bool {
        false
    }

    fn pause(&self) {}

    fn resume(&self) {}

    fn seek(&mut self, _: Duration) -> Result<(), AudioError> {
        Ok(())
    }

    fn has_source(&self) -> bool {
        true
    }
}

#[derive(Default)]
struct Labels(HashMap<String, Rectangle>);

impl Operation for Labels {
    fn traverse(&mut self, visit: &mut dyn FnMut(&mut dyn Operation)) {
        visit(self);
    }

    fn text(&mut self, _: Option<&Id>, bounds: Rectangle, label: &str) {
        self.0.insert(label.into(), bounds);
    }
}

pub(super) fn rendered_labels(
    desktop: &Desktop,
    id: window::Id,
    frame: Size,
) -> HashMap<String, Rectangle> {
    let mut view = desktop.workspace(id);
    let mut tree = Tree::new(&view);
    let renderer = Renderer::new(Font::with_name("Manrope"), Pixels(16.0));
    let widget = view.as_widget_mut();
    let node = widget.layout(&mut tree, &renderer, &layout::Limits::new(frame, frame));
    let mut labels = Labels::default();
    widget.operate(&mut tree, Layout::new(&node), &renderer, &mut labels);
    labels.0
}

#[test]
fn it_allows_long_timestamps_to_fit_in_both_player_timelines() {
    const ELAPSED: &str = "10:00:01";
    const TOTAL: &str = "11:00:05";
    const SHORT_CELL_WIDTH: f32 = 27.0;
    let (mut desktop, _) = Desktop::with_session(Ok(Session::new(LongRecording)));
    let library = desktop.windows.library_id();
    let _ = desktop.update(Message::ShowMini);
    let mini = desktop.windows.mini_id().expect("mini player opened");
    for (id, frame) in [
        (library, Size::new(850.0, 800.0)),
        (mini, Size::new(420.0, 560.0)),
    ] {
        let labels = rendered_labels(&desktop, id, frame);
        let elapsed = labels[ELAPSED];
        let total = labels[TOTAL];
        assert!(
            elapsed.width > SHORT_CELL_WIDTH,
            "elapsed time was constrained to {elapsed:?}"
        );
        assert!(
            total.width > SHORT_CELL_WIDTH,
            "total time was constrained to {total:?}"
        );
        assert!(
            elapsed.x + elapsed.width < total.x,
            "timeline labels overlapped"
        );
        assert!(
            total.x + total.width <= frame.width,
            "total time left the window"
        );
    }
}
