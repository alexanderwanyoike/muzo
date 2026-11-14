use std::path::PathBuf;

use crate::local_track::read_track;

#[test]
fn it_reads_embedded_artwork_from_a_local_track() {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../desktop/electron/infrastructure/metadata/fixtures/embedded-cover.mp3");
    let track = read_track(path);

    assert_eq!(track.title(), "embedded-cover");
    assert!(track.artwork().unwrap().starts_with(b"\x89PNG\r\n\x1a\n"));
}

#[test]
fn it_reuses_persistent_artwork_without_probing_the_source_again() {
    use crate::artwork_cache::{ArtworkCache, Revision};

    let root = tempfile::tempdir().unwrap();
    let cache = ArtworkCache::new(root.path().to_path_buf());
    let revision = Revision::new("library", "track", "/Music/song.mp3", 100, 1);
    let original = include_bytes!("../../desktop/assets/icons/muzo.png");
    let cover = cache
        .get_or_read(&revision, || Some(original.to_vec()))
        .unwrap();
    assert_eq!(&cover[..8], b"\x89PNG\r\n\x1a\n");

    let reopened = ArtworkCache::new(root.path().to_path_buf());
    assert_eq!(
        reopened.get_or_read(&revision, || panic!("album artwork was probed again")),
        Some(cover)
    );
    let updated = Revision::new("library", "track", "/Music/song.mp3", 100, 2);
    assert_eq!(reopened.get_or_read(&updated, || None), None);
}

#[test]
fn it_retries_after_a_temporary_cover_probe_failure() {
    use crate::artwork_cache::{ArtworkCache, Revision};
    let root = tempfile::tempdir().unwrap();
    let cache = ArtworkCache::new(root.path().to_path_buf());
    let revision = Revision::new("library", "track", "/Music/song.mp3", 100, 1);
    assert!(cache.get_or_read(&revision, || None).is_none());
    assert!(
        cache
            .get_or_read(&revision, || {
                Some(include_bytes!("../../desktop/assets/icons/muzo.png").to_vec())
            })
            .is_some()
    );
}
