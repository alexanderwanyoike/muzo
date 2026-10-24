import type { LibraryRemovalOutcome } from "../interfaces/library-removal";
import type { RemoveLibraryService } from "../remove-library-service";
import { failure, type Result } from "../result";
import type { CommandHandler } from "./command-handler";

export class RemoveLibraryCommand implements CommandHandler {
  readonly command = "remove_library";

  constructor(private readonly libraries: RemoveLibraryService) {}

  async handle(args: unknown): Promise<Result<LibraryRemovalOutcome>> {
    const input =
      args && typeof args === "object" && "input" in args ? args.input : null;
    if (
      !input ||
      typeof input !== "object" ||
      !("libraryId" in input) ||
      typeof input.libraryId !== "string" ||
      !input.libraryId.trim()
    ) {
      return failure("invalidInput", "Choose a source to remove.");
    }
    return this.libraries.remove(input.libraryId);
  }
}
