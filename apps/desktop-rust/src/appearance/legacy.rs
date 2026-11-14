use std::{fs, path::Path};

use rusty_leveldb::{DB, LdbIterator, Options};

use super::{Appearance, AppearanceError};

const APPEARANCE_KEY: &[u8] = b"muzo.appearance.v1";

pub fn read(path: &Path) -> Result<Option<Appearance>, AppearanceError> {
    if !path.join("CURRENT").is_file() {
        return Ok(None);
    }
    let copy = tempfile::tempdir()?;
    for entry in fs::read_dir(path)? {
        let entry = entry?;
        if entry.file_type()?.is_file() && entry.file_name() != "LOCK" {
            fs::copy(entry.path(), copy.path().join(entry.file_name()))?;
        }
    }
    let mut db = DB::open(
        copy.path(),
        Options {
            create_if_missing: false,
            ..Default::default()
        },
    )
    .map_err(|error| AppearanceError::Legacy(error.to_string()))?;
    let mut iterator = db
        .new_iter()
        .map_err(|error| AppearanceError::Legacy(error.to_string()))?;
    while let Some((key, value)) = iterator.next() {
        if !key.starts_with(b"_")
            || !key.ends_with(APPEARANCE_KEY)
            || !key
                .get(..key.len() - APPEARANCE_KEY.len())
                .is_some_and(|prefix| prefix.ends_with(b"\0\x01"))
        {
            continue;
        }
        let Some((&encoding, bytes)) = value.split_first() else {
            continue;
        };
        let decoded = match encoding {
            1 => bytes
                .iter()
                .map(|byte| char::from(*byte))
                .collect::<String>(),
            0 if bytes.len() % 2 == 0 => {
                let units = bytes
                    .chunks_exact(2)
                    .map(|word| u16::from_le_bytes([word[0], word[1]]))
                    .collect::<Vec<_>>();
                String::from_utf16(&units)
                    .map_err(|error| AppearanceError::Legacy(error.to_string()))?
            }
            _ => return Err(AppearanceError::Invalid),
        };
        return Ok(Some(serde_json::from_str(&decoded)?));
    }
    Ok(None)
}
