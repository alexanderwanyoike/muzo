import type { LibraryRemover, LibraryWatchStopper } from "./interfaces/library-removal";
import { failure, success, type Result } from "./result";

export class RemoveLibraryService {
  constructor(private readonly libraries: LibraryRemover, private readonly watcher: LibraryWatchStopper) {}

  async remove(libraryId: string): Promise<Result<void>> {
    try {
      await this.watcher.unwatch(libraryId);
      return await this.libraries.remove(libraryId)
        ? success(undefined)
        : failure("libraryNotFound", "This source has already been removed.");
    } catch {
      return failure("repository", "Could not remove the source. Try again.");
    }
  }
}
