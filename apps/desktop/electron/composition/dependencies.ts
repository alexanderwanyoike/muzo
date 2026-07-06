import { ulid } from "ulid";

import type { CommandDependencies } from "../application/commands/command-handler";
import { SqliteLibraryRepository } from "../infrastructure/sqlite/sqlite-library-repository";
import { SqliteTrackRepository } from "../infrastructure/sqlite/sqlite-track-repository";
import { muzoDatabasePath } from "../paths";

export function createCommandDependencies(): CommandDependencies {
  return {
    libraries: new SqliteLibraryRepository(muzoDatabasePath()),
    tracks: new SqliteTrackRepository(muzoDatabasePath()),
    generateLibraryId: ulid,
  };
}
