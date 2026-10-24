import type { LibraryRemovalOutcome } from "../electron/application/interfaces/library-removal";
import { failure, type Result } from "../shared/result";
import { invokeCommand } from "./runtime-bridge";

export async function removeLibrary(
  libraryId: string,
): Promise<Result<LibraryRemovalOutcome>> {
  try {
    return await invokeCommand<Result<LibraryRemovalOutcome>>(
      "remove_library",
      {
        input: { libraryId },
      },
    );
  } catch {
    return failure("repository", "Could not remove the source. Try again.");
  }
}
