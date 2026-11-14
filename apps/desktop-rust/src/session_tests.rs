use std::{num::NonZero, path::PathBuf};

use rodio::mixer;

use crate::{
    playback::Playback,
    session::{Session, SessionError},
    track::Track,
};

#[test]
fn it_advances_in_queue_order_and_survives_a_bad_file() {
    let (mixer, mut output) = mixer::mixer(NonZero::new(2).unwrap(), NonZero::new(44_100).unwrap());
    let audio = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/tone.mp3");
    let mut session = Session::new(Playback::new(mixer));
    session.extend([
        Track::read(audio.clone()),
        Track::read(audio.with_file_name("missing.mp3")),
        Track::read(audio),
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
