use std::path::Path;

use muzo_desktop_lib::application::scan_library::{AudioMetadataReader, MetadataError};
use muzo_desktop_lib::infrastructure::lofty_metadata_reader::LoftyMetadataReader;

#[test]
fn read_returns_io_error_for_a_missing_file() {
    let reader = LoftyMetadataReader::new();
    let result = reader.read(Path::new("/no/such/file.mp3"));
    assert!(
        matches!(result, Err(MetadataError::Io(_))),
        "missing file should surface as io error"
    );
}
