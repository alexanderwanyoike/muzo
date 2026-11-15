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
    changes: Vec<(String, Option<Revision>)>,
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

    pub(in crate::desktop) fn reconcile(
        &mut self,
        paths: Option<CataloguePaths>,
        covers: &Covers,
    ) -> Task<Message> {
        let Some(paths) = paths else {
            return Task::none();
        };
        let cached: Vec<_> = covers
            .keys()
            .filter_map(|id| Some((id.clone(), self.versions.get(id)?.clone())))
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
                    .unwrap_or_default()
            },
            move |changes| {
                Message::QueueArtworkReconciled(QueueReconciled {
                    generation,
                    changes,
                })
            },
        )
    }

    pub(in crate::desktop) fn apply(&mut self, result: QueueReconciled, covers: &mut Covers) {
        if result.generation != self.generation {
            return;
        }
        for (id, revision) in result.changes {
            covers.remove(&id);
            if let Some(revision) = revision {
                if let Some(version) = self.versions.get_mut(&id) {
                    version.artwork = MediaVersion::Confirmed(revision);
                }
            } else {
                self.versions.remove(&id);
            }
        }
    }
}

fn changed_versions(
    paths: &CataloguePaths,
    cached: Vec<(String, QueuedVersion)>,
) -> Vec<(String, Option<Revision>)> {
    let Ok(catalogue) = paths.open() else {
        return Vec::new();
    };
    cached
        .into_iter()
        .filter_map(|(id, old)| {
            let previous = match old.artwork {
                MediaVersion::Snapshot {
                    path,
                    size,
                    modified,
                } => Revision::new(&old.library, &id, &path.to_string_lossy(), size, modified),
                MediaVersion::Confirmed(revision) => revision,
            };
            match catalogue.artwork_revision(&old.library, &id) {
                Ok(current) if current != previous => Some((id, Some(current))),
                Err(CatalogueError::Database(rusqlite::Error::QueryReturnedNoRows)) => {
                    Some((id, None))
                }
                _ => None,
            }
        })
        .collect()
}
