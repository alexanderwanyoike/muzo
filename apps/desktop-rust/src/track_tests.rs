use std::path::PathBuf;

use crate::track::Track;

#[test]
fn it_reads_embedded_artwork_from_a_local_track() {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../desktop/electron/infrastructure/metadata/fixtures/embedded-cover.mp3");
    let track = Track::read(path);

    assert_eq!(track.title(), "embedded-cover");
    assert!(track.artwork().unwrap().starts_with(b"\x89PNG\r\n\x1a\n"));
}
