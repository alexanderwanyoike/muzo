use std::{
    fs,
    io::{self, Cursor, Read, Write},
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
const PNG_SIGNATURE: &[u8] = b"\x89PNG\r\n\x1a\n";
const MAX_CACHE_RECORD: u64 = 1_048_576;
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
        if let Some(cached) = self.lookup(revision) {
            return Some(cached);
        }
        let cover = thumbnail(&read()?)?;
        if let Err(error) = self.store(&target, &cover) {
            eprintln!("Could not cache album artwork: {error}");
        }
        Some(cover)
    }

    pub fn lookup(&self, revision: &Revision) -> Option<Vec<u8>> {
        let target = self.directory.join(format!("{}.{ENTRY_EXT}", revision.0));
        let mut record = Vec::new();
        fs::File::open(&target)
            .ok()?
            .take(MAX_CACHE_RECORD + 1)
            .read_to_end(&mut record)
            .ok()?;
        if record.len() as u64 > MAX_CACHE_RECORD {
            return None;
        }
        let (digest, bytes) = record.split_at_checked(DIGEST_BYTES)?;
        if !bytes.starts_with(PNG_SIGNATURE) || Sha256::digest(bytes).as_slice() != digest {
            return None;
        }
        let _ = fs::File::options()
            .write(true)
            .open(&target)
            .and_then(|file| file.set_modified(SystemTime::now()));
        Some(bytes.to_vec())
    }

    fn store(&self, path: &Path, bytes: &[u8]) -> io::Result<()> {
        let _guard = PUBLISH
            .lock()
            .map_err(|_| io::Error::other("Artwork cache unavailable"))?;
        let mut directory = fs::DirBuilder::new();
        directory.recursive(true);
        #[cfg(unix)]
        {
            use std::os::unix::fs::DirBuilderExt;
            directory.mode(0o700);
        }
        directory.create(&self.directory)?;
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
