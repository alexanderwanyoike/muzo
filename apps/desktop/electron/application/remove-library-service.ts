import type { LibraryRemover, LibraryWatchStopper, LibraryRemovalOutcome } from "./interfaces/library-removal";
import { failure, success, type Result } from "./result";

export class RemoveLibraryService {
  constructor(private readonly libraries: LibraryRemover, private readonly watcher: LibraryWatchStopper) {}

  async remove(libraryId: string): Promise<Result<LibraryRemovalOutcome>> {
    try {
      if (!(await this.libraries.remove(libraryId))) return failure("libraryNotFound", "This source has already been removed.");
    } catch {
      return failure("repository", "Could not remove the source. Try again.");
    }
    try {
      await this.watcher.unwatch(libraryId);
      return success({ warning: null });
    } catch {
      return success({ warning: "Source removed. Restart Muzo to release its filesystem watcher." });
    }
  }
}
