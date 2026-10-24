export interface LibraryRemover {
  remove(libraryId: string): Promise<boolean>;
}
