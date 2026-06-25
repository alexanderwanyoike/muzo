use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{mpsc, Mutex};
use std::thread::JoinHandle;
use std::time::Duration;

use notify::{Config, RecommendedWatcher, RecursiveMode, Watcher};

use crate::application::watch_filesystem_libraries::{
    FilesystemLibraryChangeHandler, FilesystemLibraryWatcher, WatchError,
};
use crate::domain::library::LibraryId;

pub struct NotifyFilesystemLibraryWatcher {
    debounce: Duration,
    registrations: Mutex<HashMap<LibraryId, WatchRegistration>>,
}

impl NotifyFilesystemLibraryWatcher {
    pub fn new(debounce: Duration) -> Self {
        Self {
            debounce,
            registrations: Mutex::new(HashMap::new()),
        }
    }
}

impl FilesystemLibraryWatcher for NotifyFilesystemLibraryWatcher {
    fn watch(
        &self,
        library_id: LibraryId,
        root: PathBuf,
        on_change: FilesystemLibraryChangeHandler,
    ) -> Result<(), WatchError> {
        let mut registrations = self
            .registrations
            .lock()
            .expect("filesystem watcher registry mutex poisoned");
        if registrations.contains_key(&library_id) {
            return Ok(());
        }

        let (tx, rx) = mpsc::channel();
        let thread_library_id = library_id.clone();
        let debounce = self.debounce;
        let worker =
            std::thread::spawn(move || debounce_events(rx, debounce, thread_library_id, on_change));

        let event_tx = tx.clone();
        let mut watcher = RecommendedWatcher::new(
            move |result: notify::Result<notify::Event>| {
                if result.is_ok() {
                    let _ = event_tx.send(WatchMessage::Event);
                }
            },
            Config::default(),
        )
        .map_err(|error| WatchError::Io(error.to_string()))?;

        if let Err(error) = watcher.watch(root.as_path(), RecursiveMode::Recursive) {
            let _ = tx.send(WatchMessage::Stop);
            return Err(WatchError::Io(error.to_string()));
        }

        registrations.insert(
            library_id,
            WatchRegistration {
                _watcher: watcher,
                tx,
                worker: Some(worker),
            },
        );

        Ok(())
    }
}

enum WatchMessage {
    Event,
    Stop,
}

fn debounce_events(
    rx: mpsc::Receiver<WatchMessage>,
    debounce: Duration,
    library_id: LibraryId,
    on_change: FilesystemLibraryChangeHandler,
) {
    while let Ok(message) = rx.recv() {
        match message {
            WatchMessage::Event => {
                if drain_event_burst(&rx, debounce) {
                    break;
                }
                on_change(library_id.clone());
            }
            WatchMessage::Stop => break,
        }
    }
}

fn drain_event_burst(rx: &mpsc::Receiver<WatchMessage>, debounce: Duration) -> bool {
    loop {
        match rx.recv_timeout(debounce) {
            Ok(WatchMessage::Event) => continue,
            Ok(WatchMessage::Stop) => return true,
            Err(mpsc::RecvTimeoutError::Timeout) => return false,
            Err(mpsc::RecvTimeoutError::Disconnected) => return true,
        }
    }
}

struct WatchRegistration {
    _watcher: RecommendedWatcher,
    tx: mpsc::Sender<WatchMessage>,
    worker: Option<JoinHandle<()>>,
}

impl Drop for WatchRegistration {
    fn drop(&mut self) {
        let _ = self.tx.send(WatchMessage::Stop);
        if let Some(worker) = self.worker.take() {
            let _ = worker.join();
        }
    }
}
