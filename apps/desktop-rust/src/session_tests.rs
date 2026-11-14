use std::{num::NonZero, path::PathBuf};

use rodio::mixer;

use crate::{
    local_track::read_track,
    playback::Playback,
    session::{Session, SessionError},
};

#[test]
fn it_advances_in_queue_order_and_survives_a_bad_file() {
    let (mixer, mut output) = mixer::mixer(NonZero::new(2).unwrap(), NonZero::new(44_100).unwrap());
    let audio = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/tone.mp3");
    let mut session = Session::new(Playback::new(mixer));
    session.extend([
        read_track(audio.clone()),
        read_track(audio.with_file_name("missing.mp3")),
        read_track(audio),
    ]);

    session.select(0).unwrap();
    output.by_ref().take(3 * 44_100 * 2).for_each(drop);
    assert!(matches!(
        session.advance_if_finished(),
        Err(SessionError::Playback(_))
    ));
    assert_eq!(session.current_index(), Some(1));
    session.next_track().unwrap();
    assert_eq!(session.current_index(), Some(2));
    assert!(
        output
            .by_ref()
            .take(1024)
            .any(|sample| sample.abs() > 0.001)
    );
}

#[test]
fn it_does_not_restart_the_current_queue_entry_when_selected_again() {
    let (mixer, mut output) = mixer::mixer(NonZero::new(2).unwrap(), NonZero::new(44_100).unwrap());
    let audio = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/tone.mp3");
    let mut session = Session::new(Playback::new(mixer));
    session.extend([read_track(audio.clone()), read_track(audio)]);
    session.select(0).unwrap();
    output.by_ref().take(22_050).for_each(drop);
    let position = session.observe().0;
    session.select(0).unwrap();
    assert_eq!(session.observe().0, position);
    session.select(1).unwrap();
    assert_eq!(session.current_index(), Some(1));
    assert!(session.observe().0 < position);
}
