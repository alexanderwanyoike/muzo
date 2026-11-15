use iced::{
    Event, event,
    keyboard::{
        self, Key, Location, Modifiers,
        key::{NativeCode, Physical},
    },
    window,
};

use super::{Desktop, Message, input_message_for_platform, queue_tests::setup};

fn press(key: &str, modifiers: Modifiers, macos: bool, mini: window::Id) -> Option<Message> {
    let key = Key::Character(key.into());
    let shortcut = Event::Keyboard(keyboard::Event::KeyPressed {
        key: key.clone(),
        modified_key: key,
        physical_key: Physical::Unidentified(NativeCode::Unidentified),
        location: Location::Standard,
        modifiers,
        text: None,
        repeat: false,
    });
    input_message_for_platform(shortcut, event::Status::Ignored, mini, macos)
}

fn mini_player() -> (Desktop, window::Id) {
    let (mut desktop, _) = setup();
    let _ = desktop.update(Message::ShowMini);
    let mini = desktop.windows.mini_id().unwrap();
    (desktop, mini)
}

#[test]
fn it_opens_the_macos_queue_with_command_shift_l() {
    let (mut desktop, mini) = mini_player();

    let message = press("l", Modifiers::LOGO | Modifiers::SHIFT, true, mini)
        .expect("Command-Shift-L should open the macOS queue");
    let _ = desktop.update(message);

    assert!(desktop.windows.queue_pending());
}

#[test]
fn it_leaves_macos_quit_lock_and_logout_chords_for_the_system() {
    let (_, mini) = mini_player();
    let command = Modifiers::LOGO;
    for modifiers in [
        command,
        command | Modifiers::CTRL,
        command | Modifiers::SHIFT,
        command | Modifiers::SHIFT | Modifiers::ALT,
    ] {
        assert!(
            press("q", modifiers, true, mini).is_none(),
            "macOS must handle {modifiers:?}+Q"
        );
    }
}

#[test]
fn it_opens_the_linux_and_windows_queue_with_control_q() {
    let (mut desktop, mini) = mini_player();

    let message =
        press("q", Modifiers::CTRL, false, mini).expect("Control-Q should open the queue");
    let _ = desktop.update(message);

    assert!(desktop.windows.queue_pending());
}
