import { watch, type FSWatcher } from "chokidar";

import type {
  FilesystemLibraryChangeHandler,
  FilesystemLibraryWatcher,
} from "../../application/interfaces/watch-interfaces";

const filesystemWatchDebounceMilliseconds = 500;

interface WatchRegistration {
  watcher: FSWatcher;
  debounce: ReturnType<typeof setTimeout> | null;
}

export class NodeFilesystemLibraryWatcher implements FilesystemLibraryWatcher {
  private readonly registrations = new Map<string, WatchRegistration>();

  async watch(
    libraryId: string,
    root: string,
    onChange: FilesystemLibraryChangeHandler,
  ): Promise<void> {
    if (this.registrations.has(libraryId)) {
      return;
    }

    const registration: WatchRegistration = {
      watcher: watch(root, {
        ignoreInitial: true,
        persistent: false,
      }),
      debounce: null,
    };

    registration.watcher.on("all", () => {
      this.scheduleChange(libraryId, registration, onChange);
    });

    this.registrations.set(libraryId, registration);
  }

  private scheduleChange(
    libraryId: string,
    registration: WatchRegistration,
    onChange: FilesystemLibraryChangeHandler,
  ): void {
    if (registration.debounce) {
      clearTimeout(registration.debounce);
    }

    registration.debounce = setTimeout(() => {
      registration.debounce = null;
      void Promise.resolve(onChange(libraryId)).catch(() => undefined);
    }, filesystemWatchDebounceMilliseconds);
    registration.debounce.unref?.();
  }
}
