use iced::{Size, window};

use super::{
    Desktop, Message,
    queue_tests::{rendered_labels, rendered_window_labels, setup},
    windows::WindowError,
};

const NO_AUDIO_OUTPUT: &str = "Could not open an audio output: Could not find any output device";
const COMPACT: Size = Size::new(420.0, 560.0);
const EXPANDED: Size = Size::new(740.0, 560.0);
const FIRST_RESIZE: u64 = 1;

fn open_mini(desktop: &mut Desktop) -> window::Id {
    let _ = desktop.update(Message::ShowMini);
    desktop.windows.mini_id().unwrap()
}

#[test]
fn it_keeps_the_audio_output_notice_while_opening_the_mini_queue() {
    let (mut desktop, _) = Desktop::with_session(Err(NO_AUDIO_OUTPUT.into()));
    assert!(
        rendered_labels(&desktop)
            .iter()
            .any(|label| label == NO_AUDIO_OUTPUT)
    );
    let mini = open_mini(&mut desktop);

    let _ = desktop.update(Message::ToggleQueue);
    let _ = desktop.update(Message::MiniResized(mini, EXPANDED));

    assert!(desktop.windows.queue_open());
    assert!(
        rendered_window_labels(&desktop, mini, EXPANDED)
            .iter()
            .any(|label| label == NO_AUDIO_OUTPUT)
    );
}

#[test]
fn it_keeps_the_audio_output_notice_when_the_native_resize_fails() {
    let (mut desktop, _) = Desktop::with_session(Err(NO_AUDIO_OUTPUT.into()));
    let mini = open_mini(&mut desktop);
    let _ = desktop.update(Message::ToggleQueue);
    let _ = desktop.update(Message::QueueResizeTimedOut(mini, FIRST_RESIZE, COMPACT));

    assert!(!desktop.windows.queue_open());
    assert!(
        rendered_window_labels(&desktop, mini, COMPACT)
            .iter()
            .any(|label| label == NO_AUDIO_OUTPUT)
    );
}

#[test]
fn it_dismisses_the_queue_resize_notice_when_trying_again() {
    let (mut desktop, _) = setup();
    let mini = open_mini(&mut desktop);
    let _ = desktop.update(Message::ToggleQueue);
    let _ = desktop.update(Message::QueueResizeTimedOut(mini, FIRST_RESIZE, COMPACT));
    let resize_error = WindowError::ResizeRejected.to_string();
    assert!(rendered_window_labels(&desktop, mini, COMPACT).contains(&resize_error));

    let _ = desktop.update(Message::ToggleQueue);
    let _ = desktop.update(Message::MiniResized(mini, EXPANDED));

    assert!(desktop.windows.queue_open());
    assert!(!rendered_window_labels(&desktop, mini, EXPANDED).contains(&resize_error));
}

#[test]
fn it_retains_a_resize_notice_until_the_mini_window_reopens() {
    let (mut desktop, _) = setup();
    let mini = open_mini(&mut desktop);
    let _ = desktop.update(Message::ToggleQueue);
    let _ = desktop.update(Message::QueueResizeTimedOut(mini, FIRST_RESIZE, COMPACT));
    let resize_error = WindowError::ResizeRejected.to_string();

    let reopened = open_mini(&mut desktop);
    assert_eq!(reopened, mini);
    assert!(rendered_window_labels(&desktop, reopened, COMPACT).contains(&resize_error));

    let _ = desktop.update(Message::ShowLibrary);
    let reopened = open_mini(&mut desktop);
    assert_ne!(reopened, mini);
    assert!(!rendered_window_labels(&desktop, reopened, COMPACT).contains(&resize_error));
}
