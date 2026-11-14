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

#[test]
fn it_applies_the_signed_opus_output_gain() {
    let open = |name| {
        let path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures")
            .join(name);
        crate::opus::OpusSource::open(std::fs::File::open(path).unwrap(), Duration::ZERO).unwrap()
    };
    let original: Vec<_> = open("tone.opus").take(4096).collect();
    let attenuated: Vec<_> = open("tone-minus-6db.opus").take(4096).collect();
    let ratio = 10.0_f32.powf(-6.0 / 20.0);
    assert!(original.iter().any(|sample| sample.abs() > 0.01));
    assert_eq!(original.len(), attenuated.len());
    assert!(
        original
            .iter()
            .zip(attenuated)
            .all(|(original, actual)| (original * ratio - actual).abs() < 0.0001)
    );
}

#[test]
fn it_warms_up_opus_before_emitting_samples_at_a_seek_target() {
    use crate::opus::OpusSource;
    use std::fs::File;

    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/tone.opus");
    let target = Duration::from_millis(1150);
    let window = 240;
    let expected: Vec<_> = OpusSource::open(File::open(&path).unwrap(), Duration::ZERO)
        .unwrap()
        .skip(55_200)
        .take(window)
        .collect();
    let sought: Vec<_> = OpusSource::open(File::open(path).unwrap(), target)
        .unwrap()
        .take(window)
        .collect();
    let error: f32 = expected
        .iter()
        .zip(&sought)
        .map(|(expected, actual)| (expected - actual).abs())
        .sum::<f32>()
        / window as f32;

    assert_eq!(expected.len(), window);
    assert_eq!(sought.len(), window);
    assert!(error < 0.003, "average sample difference was {error}");
}

#[test]
fn it_seeks_relative_to_the_beginning_of_a_cropped_opus_stream() {
    use crate::opus::OpusSource;
    use std::fs::File;

    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/tone-cropped.opus");
    let target = Duration::from_millis(100);
    let expected: Vec<_> = OpusSource::open(File::open(&path).unwrap(), Duration::ZERO)
        .unwrap()
        .skip(4_800)
        .take(240)
        .collect();
    let sought: Vec<_> = OpusSource::open(File::open(path).unwrap(), target)
        .unwrap()
        .take(240)
        .collect();
    assert_eq!(sought, expected);
}

#[test]
fn it_decodes_the_entire_opus_stream_without_trailing_or_preskip_samples() {
    use crate::opus::OpusSource;
    use std::fs::File;

    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/tone.opus");
    let track = OpusSource::open(File::open(path).unwrap(), Duration::ZERO).unwrap();

    assert_eq!(track.count(), 96_000);
}

#[test]
fn it_seeks_in_a_single_page_opus_stream_and_stops_at_the_final_granule() {
    use crate::opus::OpusSource;
    use std::fs::File;

    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/tone-short.opus");
    let decode = |milliseconds| {
        OpusSource::open(
            File::open(&path).unwrap(),
            Duration::from_millis(milliseconds),
        )
        .unwrap()
        .collect::<Vec<_>>()
    };

    let continuous = decode(0);
    assert_eq!(continuous.len(), 4_800);
    assert_eq!(decode(81), continuous[3_888..]);
    assert!(decode(100).is_empty());
}
