import { SqliteLibraryRepository } from "./sqlite-library-repository";
import { muzoDatabasePath } from "./paths";
import type { LibraryDto } from "../src/types";

interface LibraryRepository {
  list(): Promise<LibraryDto[]>;
}

interface CommandDependencies {
  libraries?: LibraryRepository;
}

export async function handleElectronCommand(
  command: string,
  _args?: unknown,
  dependencies: CommandDependencies = {},
): Promise<unknown> {
  switch (command) {
    case "list_libraries":
      return (
        dependencies.libraries ?? new SqliteLibraryRepository(muzoDatabasePath())
      ).list();
    default:
      throw new Error(`Electron command is not implemented: ${command}`);
  }
}
