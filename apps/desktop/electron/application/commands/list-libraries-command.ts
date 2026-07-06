import type { LibraryDto } from "../../../src/types";
import type { LibraryRepository } from "../interfaces/repository-interfaces";
import type { CommandHandler } from "./command-handler";

export class ListLibrariesCommand implements CommandHandler {
  readonly command = "list_libraries";

  constructor(private readonly libraries: LibraryRepository) {}

  handle(): Promise<LibraryDto[]> {
    return this.libraries.list();
  }
}
