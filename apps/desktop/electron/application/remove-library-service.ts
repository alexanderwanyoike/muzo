import type { LibraryRemover } from "./interfaces/library-removal";
import { failure, success, type Result } from "./result";

export class RemoveLibraryService {
  constructor(private readonly libraries: LibraryRemover) {}

  async remove(libraryId: string): Promise<Result<void>> {
    try {
      return await this.libraries.remove(libraryId)
        ? success(undefined)
        : failure("libraryNotFound", "This source has already been removed.");
    } catch {
      return failure("repository", "Could not remove the source. Try again.");
    }
  }
}
