use std::{
    fs,
    io::{self, Cursor, Write},
    path::{Path, PathBuf},
    sync::Mutex,
    time::SystemTime,
};

use image::{ImageFormat, ImageReader, Limits};
use sha2::{Digest, Sha256};

const BUDGET_BYTES: u64 = 128 * 1024 * 1024;
const BUDGET_ENTRIES: usize = 4_096;
const COVER_EDGE: u32 = 384;
const MAX_ARTWORK_EDGE: u32 = 8192;
const DIGEST_BYTES: usize = 32;
const ENTRY_EXT: &str = "cover";
static PUBLISH: Mutex<()> = Mutex::new(());

pub struct Revision(String);

impl Revision {
    pub fn new(library: &str, track: &str, path: &str, size: i64, modified: i64) -> Self {
        Self(format!(
            "{:x}",
            Sha256::digest(format!("{library}\0{track}\0{path}\0{size}\0{modified}"))
        ))
    }
}

pub struct ArtworkCache {
    directory: PathBuf,
}

impl ArtworkCache {
    pub fn new(directory: PathBuf) -> Self {
        Self { directory }
    }

    pub fn get_or_read(
        &self,
        revision: &Revision,
        read: impl FnOnce() -> Option<Vec<u8>>,
    ) -> Option<Vec<u8>> {
        let target = self.directory.join(format!("{}.{ENTRY_EXT}", revision.0));
        if let Ok(cached) = fs::read(&target)
            && let Some((digest, bytes)) = cached.split_at_checked(DIGEST_BYTES)
            && Sha256::digest(bytes).as_slice() == digest
        {
            let _ = fs::File::options()
                .write(true)
                .open(&target)
                .and_then(|file| file.set_modified(SystemTime::now()));
            return (!bytes.is_empty()).then(|| bytes.to_vec());
        }

        let cover = read().as_deref().and_then(thumbnail);
        if let Err(error) = self.store(&target, cover.as_deref().unwrap_or_default()) {
            eprintln!("Could not cache album artwork: {error}");
        }
        cover
    }

    fn store(&self, path: &Path, bytes: &[u8]) -> io::Result<()> {
        let _guard = PUBLISH
            .lock()
            .map_err(|_| io::Error::other("Artwork cache unavailable"))?;
        fs::create_dir_all(&self.directory)?;
        let mut file = tempfile::NamedTempFile::new_in(&self.directory)?;
        file.write_all(&Sha256::digest(bytes))?;
        file.write_all(bytes)?;
        file.persist(path).map_err(|error| error.error)?;
        self.enforce_budget()
    }

    fn enforce_budget(&self) -> io::Result<()> {
        let mut entries = fs::read_dir(&self.directory)?
            .filter_map(Result::ok)
            .filter(|entry| {
                entry
                    .path()
                    .extension()
                    .is_some_and(|extension| extension == ENTRY_EXT)
            })
            .filter_map(|entry| {
                let metadata = entry.metadata().ok()?;
                metadata.is_file().then_some((entry.path(), metadata))
            })
            .collect::<Vec<_>>();
        entries.sort_unstable_by_key(|(_, metadata)| metadata.modified().ok());
        let mut count = entries.len();
        let mut size = entries.iter().map(|(_, entry)| entry.len()).sum::<u64>();
        for (path, metadata) in entries {
            if size <= BUDGET_BYTES && count <= BUDGET_ENTRIES {
                break;
            }
            fs::remove_file(path)?;
            count -= 1;
            size -= metadata.len();
        }
        Ok(())
    }
}

pub(crate) fn thumbnail(bytes: &[u8]) -> Option<Vec<u8>> {
    let mut reader = ImageReader::new(Cursor::new(bytes))
        .with_guessed_format()
        .ok()?;
    let mut limits = Limits::default();
    limits.max_image_width = Some(MAX_ARTWORK_EDGE);
    limits.max_image_height = Some(MAX_ARTWORK_EDGE);
    limits.max_alloc = Some(128 * 1024 * 1024);
    reader.limits(limits);
    let image = reader.decode().ok()?.thumbnail(COVER_EDGE, COVER_EDGE);
    let mut output = Cursor::new(Vec::new());
    image.write_to(&mut output, ImageFormat::Png).ok()?;
    Some(output.into_inner())
}
