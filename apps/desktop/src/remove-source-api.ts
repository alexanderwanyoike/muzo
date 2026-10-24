import { failure, type Result } from "../shared/result";
import { invokeCommand } from "./runtime-bridge";

export async function removeLibrary(libraryId: string): Promise<Result<void>> {
  try {
    return await invokeCommand<Result<void>>("remove_library", { input: { libraryId } });
  } catch {
    return failure("repository", "Could not remove the source. Try again.");
  }
}
