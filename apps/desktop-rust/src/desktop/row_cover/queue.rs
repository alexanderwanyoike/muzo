use std::{collections::HashMap, path::PathBuf};

use iced::Task;

use crate::{
    artwork_cache::Revision,
    catalogue::{CatalogueError, IndexedTrack},
};

use super::{
    super::{CataloguePaths, Message},
    Covers,
};

#[derive(Clone)]
struct QueuedVersion {
    library: String,
    artwork: MediaVersion,
}

#[derive(Clone)]
enum MediaVersion {
    Snapshot {
        path: PathBuf,
        size: i64,
        modified: i64,
    },
    Confirmed(Revision),
}

#[derive(Default)]
pub(in crate::desktop) struct QueueArtwork {
    generation: u64,
    versions: HashMap<String, QueuedVersion>,
}

#[derive(Debug, Clone)]
pub struct QueueReconciled {
    generation: u64,
    changes: Result<Vec<(String, Option<Revision>)>, String>,
}

impl QueueArtwork {
    pub(in crate::desktop) fn replace<'a>(
        &mut self,
        tracks: impl IntoIterator<Item = &'a IndexedTrack>,
    ) {
        self.generation = self.generation.wrapping_add(1);
        self.versions = tracks
            .into_iter()
            .map(|track| {
                (
                    track.id.clone(),
                    QueuedVersion {
                        library: track.library_id.clone(),
                        artwork: MediaVersion::Snapshot {
                            path: track.path.clone(),
                            size: track.file_size,
                            modified: track.file_mtime,
                        },
                    },
                )
            })
            .collect();
    }

    pub(in crate::desktop) fn remove_library(&mut self, library: &str) {
        self.generation = self.generation.wrapping_add(1);
        self.versions
            .retain(|_, version| version.library != library);
    }

    pub(in crate::desktop) fn reconcile<'a>(
        &mut self,
        paths: Option<CataloguePaths>,
        track_ids: impl IntoIterator<Item = &'a str>,
    ) -> Task<Message> {
        let Some(paths) = paths else {
            return Task::none();
        };
        let cached: HashMap<_, _> = track_ids
            .into_iter()
            .filter_map(|id| Some((id.to_owned(), self.versions.get(id)?.clone())))
            .collect();
        if cached.is_empty() {
            return Task::none();
        }
        self.generation = self.generation.wrapping_add(1);
        let generation = self.generation;
        Task::perform(
            async move {
                tokio::task::spawn_blocking(move || changed_versions(&paths, cached))
                    .await
                    .map_err(|error| error.to_string())?
                    .map_err(|error| error.to_string())
            },
            move |changes| {
                Message::QueueArtworkReconciled(QueueReconciled {
                    generation,
                    changes,
                })
            },
        )
    }

    pub(in crate::desktop) fn remember(&mut self, id: &str, revision: Revision) {
        if let Some(version) = self.versions.get_mut(id) {
            version.artwork = MediaVersion::Confirmed(revision);
        }
    }

    pub(in crate::desktop) fn apply(
        &mut self,
        result: QueueReconciled,
        covers: &mut Covers,
    ) -> Vec<String> {
        if result.generation != self.generation {
            return Vec::new();
        }
        let changes = match result.changes {
            Ok(changes) => changes,
            Err(error) => {
                eprintln!("Could not check queued artwork revisions: {error}");
                return Vec::new();
            }
        };
        let mut updated = Vec::with_capacity(changes.len());
        for (id, revision) in changes {
            covers.remove(&id);
            if let Some(revision) = revision {
                self.remember(&id, revision);
            } else {
                self.versions.remove(&id);
            }
            updated.push(id);
        }
        updated
    }
}

fn changed_versions(
    paths: &CataloguePaths,
    cached: HashMap<String, QueuedVersion>,
) -> Result<Vec<(String, Option<Revision>)>, CatalogueError> {
    let catalogue = paths.open()?;
    let mut changes = Vec::new();
    for (id, old) in cached {
        let previous = match old.artwork {
            MediaVersion::Snapshot {
                path,
                size,
                modified,
            } => Revision::new(&old.library, &id, &path.to_string_lossy(), size, modified),
            MediaVersion::Confirmed(revision) => revision,
        };
        match catalogue.artwork_revision(&old.library, &id) {
            Ok(current) if current != previous => changes.push((id, Some(current))),
            Ok(_) => {}
            Err(CatalogueError::UnknownTrack) => changes.push((id, None)),
            Err(error) => return Err(error),
        }
    }
    Ok(changes)
}
