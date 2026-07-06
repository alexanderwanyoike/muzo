import { ulid } from "ulid";

import { muzoDatabasePath } from "./paths";
import { SqliteLibraryRepository } from "./sqlite-library-repository";
import { SqliteTrackRepository } from "./sqlite-track-repository";
import type { CommandDependencies } from "./command-handler";

export function createCommandDependencies(): CommandDependencies {
  return {
    libraries: new SqliteLibraryRepository(muzoDatabasePath()),
    tracks: new SqliteTrackRepository(muzoDatabasePath()),
    generateLibraryId: ulid,
  };
}
