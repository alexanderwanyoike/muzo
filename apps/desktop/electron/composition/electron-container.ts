import { ulid } from "ulid";

import { AddLibraryCommand } from "../application/commands/add-library-command";
import { ListLibrariesCommand } from "../application/commands/list-libraries-command";
import { ListTracksCommand } from "../application/commands/list-tracks-command";
import { SqliteLibraryRepository } from "../infrastructure/sqlite/sqlite-library-repository";
import { SqliteTrackRepository } from "../infrastructure/sqlite/sqlite-track-repository";
import { CommandDispatcher } from "../ipc/command-dispatcher";
import { muzoDatabasePath } from "../paths";

export interface ElectronContainerOptions {
  dbPath?: string;
  generateLibraryId?: () => string;
}

export class ElectronContainer {
  readonly commandDispatcher: CommandDispatcher;

  constructor(options: ElectronContainerOptions = {}) {
    const dbPath = options.dbPath ?? muzoDatabasePath();
    const generateLibraryId = options.generateLibraryId ?? ulid;
    const libraries = new SqliteLibraryRepository(dbPath);
    const tracks = new SqliteTrackRepository(dbPath);

    this.commandDispatcher = new CommandDispatcher([
      new AddLibraryCommand(libraries, generateLibraryId),
      new ListLibrariesCommand(libraries),
      new ListTracksCommand(tracks),
    ]);
  }
}
