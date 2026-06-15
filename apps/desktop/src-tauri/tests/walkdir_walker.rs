use std::fs;
use std::path::Path;

use muzo_desktop_lib::application::scan_library::{FilesystemWalker, WalkError};
use muzo_desktop_lib::infrastructure::walkdir_walker::{WalkdirWalker, AUDIO_EXTENSIONS};

#[test]
fn walk_returns_every_audio_file_under_the_root_recursively() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();

    fs::write(root.join("a.mp3"), b"mp3").unwrap();
    fs::create_dir_all(root.join("sub")).unwrap();
    fs::write(root.join("sub/b.flac"), b"flac").unwrap();
    fs::write(root.join("readme.txt"), b"ignore").unwrap();
    fs::write(root.join("cover.jpg"), b"ignore").unwrap();

    let walked = WalkdirWalker::new().walk_audio_files(root).unwrap();

    let paths: Vec<String> = walked
        .iter()
        .map(|f| f.path.file_name().unwrap().to_string_lossy().into_owned())
        .collect();

    assert_eq!(paths.len(), 2, "only audio files should be returned");
    assert!(paths.contains(&"a.mp3".to_string()));
    assert!(paths.contains(&"b.flac".to_string()));
}

#[test]
fn walk_populates_size_and_mtime_for_each_file() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    fs::write(root.join("a.mp3"), b"123456").unwrap();

    let walked = WalkdirWalker::new().walk_audio_files(root).unwrap();

    assert_eq!(walked.len(), 1);
    assert_eq!(walked[0].size, 6);
    assert!(walked[0].mtime > 0, "mtime should be a unix timestamp");
}

#[test]
fn walk_returns_not_found_when_the_root_does_not_exist() {
    let result = WalkdirWalker::new().walk_audio_files(Path::new("/no/such/dir"));
    assert!(matches!(result, Err(WalkError::NotFound(_))));
}

#[test]
fn audio_extensions_constant_lists_the_supported_formats() {
    assert!(AUDIO_EXTENSIONS.contains(&"mp3"));
    assert!(AUDIO_EXTENSIONS.contains(&"flac"));
    assert!(AUDIO_EXTENSIONS.contains(&"m4a"));
    assert!(AUDIO_EXTENSIONS.contains(&"ogg"));
    assert!(AUDIO_EXTENSIONS.contains(&"opus"));
    assert!(AUDIO_EXTENSIONS.contains(&"wav"));
    assert!(!AUDIO_EXTENSIONS.contains(&"txt"));
}
