use crate::application::scan_dropbox_library::{
    DropboxCatalogError, DropboxLibraryCatalog, DropboxSourceFile,
};
use crate::infrastructure::dropbox_api_client::{
    DropboxClientError, DropboxEntry, DropboxEntryTag,
};
use crate::infrastructure::walkdir_walker::AUDIO_EXTENSIONS;

pub trait DropboxFileLister {
    fn list_folder_recursive(
        &self,
        access_token: &str,
        path: &str,
    ) -> Result<Vec<DropboxEntry>, DropboxClientError>;
}

pub struct DropboxLibraryCatalogAdapter<T> {
    file_lister: T,
}

impl<T> DropboxLibraryCatalogAdapter<T> {
    pub fn new(file_lister: T) -> Self {
        Self { file_lister }
    }
}

impl<T> DropboxLibraryCatalog for DropboxLibraryCatalogAdapter<T>
where
    T: DropboxFileLister,
{
    fn list_audio_files(
        &self,
        access_token: &str,
        root: &str,
    ) -> Result<Vec<DropboxSourceFile>, DropboxCatalogError> {
        let entries = self
            .file_lister
            .list_folder_recursive(access_token, root)
            .map_err(|error| DropboxCatalogError::Io(format!("{error:?}")))?;

        entries
            .into_iter()
            .filter(|entry| entry.tag == DropboxEntryTag::File)
            .filter(is_supported_audio_file)
            .map(source_file_from_entry)
            .collect()
    }
}

impl<'a, T> DropboxFileLister for crate::infrastructure::dropbox_api_client::DropboxApiClient<'a, T>
where
    T: crate::infrastructure::dropbox_api_client::DropboxHttpTransport + ?Sized,
{
    fn list_folder_recursive(
        &self,
        access_token: &str,
        path: &str,
    ) -> Result<Vec<DropboxEntry>, DropboxClientError> {
        crate::infrastructure::dropbox_api_client::DropboxApiClient::list_folder_recursive(
            self,
            access_token,
            path,
        )
    }
}

fn is_supported_audio_file(entry: &DropboxEntry) -> bool {
    entry
        .name
        .rsplit_once('.')
        .map(|(_, extension)| AUDIO_EXTENSIONS.contains(&extension.to_lowercase().as_str()))
        .unwrap_or(false)
}

fn source_file_from_entry(entry: DropboxEntry) -> Result<DropboxSourceFile, DropboxCatalogError> {
    Ok(DropboxSourceFile {
        path: entry
            .path_display
            .or(entry.path_lower)
            .unwrap_or(entry.name),
        size: entry.size.unwrap_or_default(),
        modified_epoch_seconds: entry
            .server_modified
            .as_deref()
            .map(parse_dropbox_timestamp)
            .transpose()?
            .unwrap_or_default(),
    })
}

fn parse_dropbox_timestamp(value: &str) -> Result<u64, DropboxCatalogError> {
    let parsed = chrono::DateTime::parse_from_rfc3339(value)
        .map_err(|error| DropboxCatalogError::Io(error.to_string()))?;
    Ok(parsed.timestamp().try_into().unwrap_or_default())
}

#[cfg(test)]
mod tests {
    use super::{DropboxFileLister, DropboxLibraryCatalogAdapter};
    use crate::application::scan_dropbox_library::DropboxLibraryCatalog;
    use crate::infrastructure::dropbox_api_client::{
        DropboxClientError, DropboxEntry, DropboxEntryTag,
    };

    struct FakeDropboxFileLister {
        entries: Vec<DropboxEntry>,
    }

    impl DropboxFileLister for FakeDropboxFileLister {
        fn list_folder_recursive(
            &self,
            _access_token: &str,
            _path: &str,
        ) -> Result<Vec<DropboxEntry>, DropboxClientError> {
            Ok(self.entries.clone())
        }
    }

    #[test]
    fn lists_supported_dropbox_audio_files_as_source_files() {
        let adapter = DropboxLibraryCatalogAdapter::new(FakeDropboxFileLister {
            entries: vec![
                DropboxEntry {
                    tag: DropboxEntryTag::Folder,
                    name: "Album".into(),
                    id: "id:folder".into(),
                    path_lower: Some("/music/album".into()),
                    path_display: Some("/Music/Album".into()),
                    server_modified: None,
                    rev: None,
                    size: None,
                    content_hash: None,
                },
                DropboxEntry {
                    tag: DropboxEntryTag::File,
                    name: "Song.MP3".into(),
                    id: "id:file-1".into(),
                    path_lower: Some("/music/album/song.mp3".into()),
                    path_display: Some("/Music/Album/Song.MP3".into()),
                    server_modified: Some("2026-06-25T10:30:00Z".into()),
                    rev: Some("rev-1".into()),
                    size: Some(12_345),
                    content_hash: Some("hash-1".into()),
                },
                DropboxEntry {
                    tag: DropboxEntryTag::File,
                    name: "Cover.jpg".into(),
                    id: "id:file-2".into(),
                    path_lower: Some("/music/album/cover.jpg".into()),
                    path_display: Some("/Music/Album/Cover.jpg".into()),
                    server_modified: Some("2026-06-25T10:31:00Z".into()),
                    rev: Some("rev-2".into()),
                    size: Some(999),
                    content_hash: Some("hash-2".into()),
                },
            ],
        });

        let files = adapter
            .list_audio_files("access-123", "/Music")
            .expect("Dropbox listing should map");

        assert_eq!(files.len(), 1);
        assert_eq!(files[0].path, "/Music/Album/Song.MP3");
        assert_eq!(files[0].size, 12_345);
        assert_eq!(files[0].modified_epoch_seconds, 1_782_383_400);
    }
}
