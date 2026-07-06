import {
  asClass,
  asFunction,
  asValue,
  createContainer,
  InjectionMode,
  type AwilixContainer,
} from "awilix";
import { ulid } from "ulid";

import { AddLibraryCommand } from "../application/commands/add-library-command";
import type { CommandHandler } from "../application/commands/command-handler";
import { ListLibrariesCommand } from "../application/commands/list-libraries-command";
import { ListTracksCommand } from "../application/commands/list-tracks-command";
import type {
  LibraryRepository,
  TrackRepository,
} from "../application/interfaces/repository-interfaces";
import { SqliteLibraryRepository } from "../infrastructure/sqlite/sqlite-library-repository";
import { runSqliteMigrations } from "../infrastructure/sqlite/sqlite-migrations";
import { SqliteTrackRepository } from "../infrastructure/sqlite/sqlite-track-repository";
import { CommandDispatcher } from "../ipc/command-dispatcher";
import { muzoDatabasePath } from "../paths";

export interface ElectronContainerOptions {
  dbPath?: string;
  generateLibraryId?: () => string;
}

export interface ElectronCradle {
  dbPath: string;
  generateLibraryId: () => string;
  libraries: LibraryRepository;
  tracks: TrackRepository;
  addLibraryCommand: AddLibraryCommand;
  listLibrariesCommand: ListLibrariesCommand;
  listTracksCommand: ListTracksCommand;
  commandHandlers: CommandHandler[];
  commandDispatcher: CommandDispatcher;
  migrateDatabase: () => Promise<void>;
}

export function createElectronContainer(
  options: ElectronContainerOptions = {},
): AwilixContainer<ElectronCradle> {
  const container = createContainer<ElectronCradle>({
    injectionMode: InjectionMode.CLASSIC,
  });

  container.register({
    dbPath: asValue(options.dbPath ?? muzoDatabasePath()),
    generateLibraryId: asValue(options.generateLibraryId ?? ulid),
    migrateDatabase: asFunction(
      (dbPath: string) => () => runSqliteMigrations(dbPath),
    ).singleton(),
    libraries: asClass(SqliteLibraryRepository).singleton(),
    tracks: asClass(SqliteTrackRepository).singleton(),
    addLibraryCommand: asClass(AddLibraryCommand).singleton(),
    listLibrariesCommand: asClass(ListLibrariesCommand).singleton(),
    listTracksCommand: asClass(ListTracksCommand).singleton(),
    commandHandlers: asFunction(
      (
        addLibraryCommand: AddLibraryCommand,
        listLibrariesCommand: ListLibrariesCommand,
        listTracksCommand: ListTracksCommand,
      ) => [
        addLibraryCommand,
        listLibrariesCommand,
        listTracksCommand,
      ],
    ).singleton(),
    commandDispatcher: asClass(CommandDispatcher).singleton(),
  });

  return container;
}
