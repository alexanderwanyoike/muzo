import type { LibraryRecord } from "../interfaces/library-record";
import type { LibraryRepository } from "../interfaces/repository-interfaces";
import type { CommandHandler } from "./command-handler";
import { repositoryResult, type Result } from "../result";

export class ListLibrariesCommand implements CommandHandler {
  readonly command = "list_libraries";

  constructor(private readonly libraries: LibraryRepository) {}

  handle(): Promise<Result<LibraryRecord[]>> { return repositoryResult(() => this.libraries.list()); }
}
