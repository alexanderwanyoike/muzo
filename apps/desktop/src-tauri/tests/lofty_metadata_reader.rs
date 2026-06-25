use std::path::Path;

use muzo_desktop_lib::application::scan_library::{AudioMetadataReader, MetadataError};
use muzo_desktop_lib::infrastructure::lofty_metadata_reader::LoftyMetadataReader;

#[test]
fn read_returns_id3_metadata_from_an_mp3_file() {
    let temp_dir = tempfile::tempdir().unwrap();
    let file_path = temp_dir.path().join("tagged.mp3");
    std::fs::write(&file_path, hex_bytes(TAGGED_MP3_HEX)).unwrap();

    let reader = LoftyMetadataReader::new();
    let metadata = reader.read(&file_path).unwrap();

    assert_eq!(metadata.title, "Tagged Title");
    assert_eq!(metadata.artist, "Tagged Artist");
    assert_eq!(metadata.album, Some("Tagged Album".into()));
    assert_eq!(metadata.track_number, Some(7));
    assert_eq!(metadata.disc_number, Some(2));
    assert_eq!(metadata.genre, Some("Rock".into()));
    assert_eq!(metadata.year, Some(1999));
}

#[test]
fn read_returns_io_error_for_a_missing_file() {
    let reader = LoftyMetadataReader::new();
    let result = reader.read(Path::new("/no/such/file.mp3"));
    assert!(
        matches!(result, Err(MetadataError::Io(_))),
        "missing file should surface as io error"
    );
}

fn hex_bytes(hex: &str) -> Vec<u8> {
    hex.as_bytes()
        .chunks_exact(2)
        .map(|chunk| {
            let byte = std::str::from_utf8(chunk).unwrap();
            u8::from_str_radix(byte, 16).unwrap()
        })
        .collect()
}

const TAGGED_MP3_HEX: &str = "\
4944330400000000012b544954320000000e000003546167676564205469746c6500545045310000000f000003546167676564204172746973740054414c420000000e00000354616767656420416c62756d005452434b00000006000003372f31320054504f5300000005000003322f330054434f4e00000006000003526f636b0054445243000000060000033139393900545353450000000f0000034c61766636302e31362e3130300000000000000000000000ffe338c0000000000000000000496e666f0000000f00000004000001f800929292929292929292929292929292929292929292929292b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6b6dbdbdbdbdbdbdbdbdbdbdbdbdbdbdbdbdbdbdbdbdbdbdbdbdbffffffffffffffffffffffffffffffffffffffffffffffffff000000004c61766336302e33310000000000000000000000002403a000000000000001f8411ad5210000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000ffe318c40000000348000000004c414d45332e3130305555555555555555555555555555555555555555555555555555555555555555555555555555555555555555555555555555ffe318c43b00000348000000005555555555555555555555555555555555555555555555555555555555555555555555555555555555555555555555555555555555555555555555ffe318c47600000348000000005555555555555555555555555555555555555555555555555555555555555555555555555555555555555555555555555555555555555555555555555555ffe318c4b100000348000000005555555555555555555555555555555555555555555555555555555555555555555555555555555555555555555555555555555555555555555555";
