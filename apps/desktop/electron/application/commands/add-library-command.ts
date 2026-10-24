import type { AddLibraryInput, LibraryRecord } from "../interfaces/library-record";
import type { LibraryRepository } from "../interfaces/repository-interfaces";
import type { AddedLibraryWatcher } from "../watch-filesystem-libraries-service";
import type { LibraryScanner } from "../scan-library-service";
import type { Logger } from "../interfaces/logger-interfaces";
import type { CommandHandler } from "./command-handler";

export class AddLibraryCommand implements CommandHandler {
  readonly command = "add_library";

  constructor(
    private readonly libraries: LibraryRepository,
    private readonly generateLibraryId: () => string,
    private readonly watchFilesystemLibrariesService: AddedLibraryWatcher,
    private readonly scanner: LibraryScanner,
    private readonly logger: Logger,
  ) {}

  async handle(args: unknown): Promise<LibraryRecord> {
    const input = parseAddLibraryArgs(args);

    if (input.name.trim().length === 0) {
      throw { kind: "emptyName" };
    }

    if (input.location.trim().length === 0) {
      throw { kind: "emptyLocation" };
    }

    const library: LibraryRecord = {
      id: this.generateLibraryId(),
      name: input.name,
      kind: input.kind,
      location: input.location,
    };
    await this.libraries.add(library);
    await this.watchFilesystemLibrariesService.watchLibrary(library);
    if (library.kind === "filesystem") {
      try {
        const scan = await this.scanner.scan(library.id);
        if (!scan.ok) this.logger.error(`initial library scan failed for ${library.id}: ${scan.error.message}`);
      }
      catch (error) {
        this.logger.error(`initial library scan failed for ${library.id}: ${error instanceof Error ? error.message : "Check the folder and refresh in Settings."}`);
      }
    }
    return library;
  }
}

function parseAddLibraryArgs(args: unknown): AddLibraryInput {
  if (!args || typeof args !== "object" || !("input" in args)) {
    throw { kind: "repository", message: "missing add_library input" };
  }
  const input = (args as { input: unknown }).input;
  if (!input || typeof input !== "object") {
    throw { kind: "repository", message: "invalid add_library input" };
  }
  return input as AddLibraryInput;
}
