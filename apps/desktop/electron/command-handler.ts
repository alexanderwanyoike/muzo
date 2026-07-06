import type { LibraryDto } from "../src/types";

export interface LibraryRepository {
  add(library: LibraryDto): Promise<void>;
  list(): Promise<LibraryDto[]>;
}

export interface CommandDependencies {
  libraries: LibraryRepository;
  generateLibraryId: () => string;
}

export interface CommandHandler {
  readonly command: string;
  handle(args: unknown, dependencies: CommandDependencies): Promise<unknown>;
}
