import { ulid } from "ulid";

import { muzoDatabasePath } from "./paths";
import { SqliteLibraryRepository } from "./sqlite-library-repository";
import type { CommandDependencies } from "./command-handler";

export function createCommandDependencies(): CommandDependencies {
  return {
    libraries: new SqliteLibraryRepository(muzoDatabasePath()),
    generateLibraryId: ulid,
  };
}
