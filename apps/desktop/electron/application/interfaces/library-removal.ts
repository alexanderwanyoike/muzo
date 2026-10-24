export interface LibraryRemover {
  remove(libraryId: string): Promise<boolean>;
}

export interface LibraryWatchStopper {
  unwatch(libraryId: string): Promise<void>;
}

export interface LibraryRemovalOutcome {
  warning: string | null;
}
