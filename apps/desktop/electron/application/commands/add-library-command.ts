import type { LibraryRecord } from "../interfaces/library-record";
import type { LibraryRepository } from "../interfaces/repository-interfaces";
import type { AddedLibraryWatcher } from "../watch-filesystem-libraries-service";
import type { LibraryScanner } from "../scan-library-service";
import type { Logger } from "../interfaces/logger-interfaces";
import type { CommandHandler } from "./command-handler";
import { failure, repositoryResult, success, type Result } from "../result";
import { parseInput } from "./command-arguments";

export class AddLibraryCommand implements CommandHandler {
  readonly command = "add_library";
  constructor(
    private readonly libraries: LibraryRepository,
    private readonly generateLibraryId: () => string,
    private readonly watcher: AddedLibraryWatcher,
    private readonly scanner: LibraryScanner,
    private readonly logger: Logger,
  ) {}

  async handle(args: unknown): Promise<Result<LibraryRecord>> {
    const input = parseInput(args, this.command);
    if (!input.ok) return input;
    const { name, kind, location } = input.value;
    if (typeof name !== "string" || typeof location !== "string" || (kind !== "filesystem" && kind !== "dropbox")) return failure("repository", "invalid add_library input");
    if (!name.trim()) return failure("emptyName", "Library name must not be empty.");
    if (!location.trim()) return failure("emptyLocation", "Library location must not be empty.");
    const library: LibraryRecord = { id: this.generateLibraryId(), name, kind, location };
    const saved = await repositoryResult(() => this.libraries.add(library));
    if (!saved.ok) return saved;
    const watchFailure = await this.watcher.watchLibrary(library);
    if (watchFailure) this.logger.error(`initial library watch failed for ${library.id}: ${watchFailure.message}`);
    if (library.kind === "filesystem") {
      const scan = await this.scanner.scan(library.id);
      if (!scan.ok) this.logger.error(`initial library scan failed for ${library.id}: ${scan.error.message}`);
    }
    return success(library);
  }
}
