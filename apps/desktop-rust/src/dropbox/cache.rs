use std::{
    fs,
    io::{self, Read},
    path::{Path, PathBuf},
};

use sha2::{Digest, Sha256};

use super::{Client, DropboxError};

const AUDIO_CACHE_BYTES: u64 = 1024 * 1024 * 1024;

pub fn download(
    client: &Client,
    token: &str,
    root: &Path,
    account: &str,
    path: &str,
) -> Result<PathBuf, DropboxError> {
    fs::create_dir_all(root)?;
    let hash = format!("{:x}", Sha256::digest(format!("{account}\0{path}")));
    let extension = Path::new(path)
        .extension()
        .and_then(|extension| extension.to_str())
        .unwrap_or("audio");
    let mut buffer = tempfile::Builder::new()
        .prefix(".download-")
        .tempfile_in(root)?;
    let response = client.download(token, path)?;
    let copied = io::copy(
        &mut response.take(AUDIO_CACHE_BYTES + 1),
        buffer.as_file_mut(),
    )?;
    if copied > AUDIO_CACHE_BYTES {
        return Err(DropboxError::CacheLimit);
    }
    let mut existing = fs::read_dir(root)?
        .filter_map(Result::ok)
        .filter(|entry| !entry.file_name().to_string_lossy().starts_with('.'))
        .filter_map(|entry| {
            entry
                .metadata()
                .ok()
                .filter(|stat| stat.is_file())
                .map(|stat| (entry.path(), stat))
        })
        .collect::<Vec<_>>();
    existing.sort_by_key(|(_, stat)| stat.modified().ok());
    let mut total = existing
        .iter()
        .fold(copied, |total, (_, stat)| total.saturating_add(stat.len()));
    for (file, stat) in existing {
        if total <= AUDIO_CACHE_BYTES {
            break;
        }
        if fs::remove_file(file).is_ok() {
            total -= stat.len();
        }
    }
    if total > AUDIO_CACHE_BYTES {
        return Err(DropboxError::CacheLimit);
    }
    let target = root.join(format!(
        "{hash}-{}.{extension}",
        uuid::Uuid::new_v4().simple()
    ));
    buffer.persist(&target).map_err(|error| error.error)?;
    Ok(target)
}
