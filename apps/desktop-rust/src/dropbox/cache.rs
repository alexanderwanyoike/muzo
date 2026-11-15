use std::{
    fs,
    io::{self, Read},
    path::{Path, PathBuf},
};

use sha2::{Digest, Sha256};

use super::{Client, DropboxError};

static CACHE_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

const AUDIO_CACHE_BYTES: u64 = 1024 * 1024 * 1024;
const METADATA_HEADER: &str = "Dropbox-API-Result";

#[derive(serde::Deserialize)]
pub struct DownloadMetadata {
    pub rev: String,
    pub size: u64,
}

pub struct Download {
    pub path: PathBuf,
    pub metadata: Option<DownloadMetadata>,
}

pub fn download(
    client: &Client,
    token: &str,
    root: &Path,
    account: &str,
    path: &str,
) -> Result<Download, DropboxError> {
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
    let metadata = response.headers().get(METADATA_HEADER).and_then(|header| {
        match serde_json::from_slice::<DownloadMetadata>(header.as_bytes()) {
            Ok(metadata) => Some(metadata),
            Err(error) => {
                eprintln!("Could not read Dropbox audio revision: {error}");
                None
            }
        }
    });
    let copied = io::copy(
        &mut response.take(AUDIO_CACHE_BYTES + 1),
        buffer.as_file_mut(),
    )?;
    if copied > AUDIO_CACHE_BYTES {
        return Err(DropboxError::CacheLimit);
    }
    let _publish = CACHE_LOCK
        .lock()
        .map_err(|_| std::io::Error::other("Dropbox audio cache is unavailable"))?;
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
    Ok(Download {
        path: target,
        metadata: metadata.filter(|metadata| metadata.size == copied),
    })
}
