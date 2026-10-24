export type FilesystemLibraryChangeHandler = (
  libraryId: string,
) => Promise<void> | void;

export interface FilesystemLibraryWatcher {
  watch(
    libraryId: string,
    root: string,
    onChange: FilesystemLibraryChangeHandler,
  ): Promise<void>;
}
