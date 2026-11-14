use std::{num::NonZero, path::PathBuf};

use rodio::mixer;

use crate::{
    desktop::{Desktop, Message},
    local_track::read_track,
    playback::Playback,
    session::Session,
};

#[test]
fn it_keeps_the_same_audio_position_when_switching_between_players() {
    let (mixer, mut output) = mixer::mixer(NonZero::new(2).unwrap(), NonZero::new(44_100).unwrap());
    let mut session = Session::new(Playback::new(mixer));
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/tone.mp3");
    session.extend([read_track(path)]);
    let (mut desktop, _) = Desktop::with_session(Ok(session));
    let _ = desktop.update(Message::Select(0));
    output.by_ref().take(44_100).for_each(drop);
    let (_, before) = desktop.now_playing().unwrap();

    let _ = desktop.update(Message::ShowMini);
    let _ = desktop.update(Message::ToggleQueue);
    let _ = desktop.update(Message::ShowLibrary);
    output.by_ref().take(8_820).for_each(drop);

    let (index, after) = desktop.now_playing().unwrap();
    assert_eq!(index, 0);
    assert!(after > before);
}
