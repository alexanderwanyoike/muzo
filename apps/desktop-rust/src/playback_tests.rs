use std::{num::NonZero, path::Path, time::Duration};

use rodio::mixer;

use crate::{playback::Playback, session::AudioPlayback};

#[test]
fn it_plays_real_local_audio_and_can_pause_seek_and_resume() {
    let (mixer, mut samples) =
        mixer::mixer(NonZero::new(2).unwrap(), NonZero::new(44_100).unwrap());
    let mut playback = Playback::new(mixer);
    let file = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/tone.mp3");

    playback.play(&file).unwrap();
    assert!(!playback.is_paused());
    assert!(playback.duration() > Duration::ZERO);
    assert!(
        samples
            .by_ref()
            .take(4_410)
            .any(|sample| sample.abs() > 0.001)
    );

    playback.pause();
    assert!(playback.is_paused());

    let target = playback.duration() / 2;
    std::thread::scope(|scope| {
        let seek = scope.spawn(|| playback.seek(target));
        let deadline = std::time::Instant::now() + Duration::from_secs(2);
        while !seek.is_finished() && std::time::Instant::now() < deadline {
            assert!(samples.next().is_some());
            std::thread::yield_now();
        }
        seek.join().unwrap().unwrap();
    });

    playback.resume();
    assert!(!playback.is_paused());
    samples.by_ref().take(4_410).for_each(drop);
    assert!(playback.position() >= target);
}

#[test]
fn it_can_seek_before_the_output_device_pulls_its_first_sample() {
    let (mixer, _output) = mixer::mixer(NonZero::new(2).unwrap(), NonZero::new(44_100).unwrap());
    let file = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/tone.mp3");
    let (send, receive) = std::sync::mpsc::channel();
    let worker = std::thread::spawn(move || {
        let mut playback = Playback::new(mixer);
        playback.play(&file).unwrap();
        let target = playback.duration() / 2;
        let result = playback.seek(target).map(|()| playback.position());
        send.send((result, target)).ok();
    });
    let (position, target) = receive.recv_timeout(Duration::from_secs(1)).unwrap();
    assert_eq!(position.unwrap(), target);
    worker.join().unwrap();
}

#[test]
fn it_plays_and_seeks_ogg_opus() {
    let (mixer, mut samples) =
        mixer::mixer(NonZero::new(2).unwrap(), NonZero::new(48_000).unwrap());
    let file = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/tone.opus");
    let mut playback = Playback::new(mixer);
    playback.play(&file).unwrap();
    let target = playback.duration() / 2;
    assert!(target > Duration::ZERO);
    playback.seek(target).unwrap();
    assert!(
        samples
            .by_ref()
            .take(4096)
            .any(|sample| sample.abs() > 0.001)
    );
}
