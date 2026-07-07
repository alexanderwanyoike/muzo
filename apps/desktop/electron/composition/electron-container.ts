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
import {
  ListTrackPlayCountsCommand,
  RecordTrackPlayCommand,
} from "../application/commands/play-history-command";
import {
  AddTrackToPlaylistCommand,
  CreatePlaylistCommand,
  ListPlaylistsCommand,
  RemovePlaylistEntryCommand,
  ReorderPlaylistEntriesCommand,
} from "../application/commands/playlist-command";
import { PrepareTrackAudioSourceCommand } from "../application/commands/prepare-track-audio-source-command";
import { ScanLibraryCommand } from "../application/commands/scan-library-command";
import {
  ClearTrackMetadataOverrideCommand,
  EditTrackMetadataCommand,
} from "../application/commands/track-metadata-command";
import type {
  LibraryRepository,
  TrackRepository,
} from "../application/interfaces/repository-interfaces";
import type { PlayHistoryRepository } from "../application/interfaces/play-history-interfaces";
import type { PlaylistRepository } from "../application/interfaces/playlist-interfaces";
import type { AudioSourceRegistry } from "../application/interfaces/audio-source-interfaces";
import type {
  AudioFileWalker,
  AudioMetadataReader,
} from "../application/interfaces/scan-interfaces";
import { PrepareTrackAudioSourceApplicationService } from "../application/prepare-track-audio-source-service";
import { PlaylistApplicationService } from "../application/playlist-service";
import { ReconcileFilesystemLibrariesService } from "../application/reconcile-filesystem-libraries-service";
import { NodeAudioStreamServer } from "../infrastructure/audio/node-audio-stream-server";
import { ScanLibraryService } from "../application/scan-library-service";
import { NodeAudioFileWalker } from "../infrastructure/filesystem/node-audio-file-walker";
import { MusicMetadataReader } from "../infrastructure/metadata/music-metadata-reader";
import { SqliteLibraryRepository } from "../infrastructure/sqlite/sqlite-library-repository";
import { runSqliteMigrations } from "../infrastructure/sqlite/sqlite-migrations";
import { SqlitePlayHistoryRepository } from "../infrastructure/sqlite/sqlite-play-history-repository";
import { SqlitePlaylistRepository } from "../infrastructure/sqlite/sqlite-playlist-repository";
import { SqliteTrackRepository } from "../infrastructure/sqlite/sqlite-track-repository";
import { CommandDispatcher } from "../ipc/command-dispatcher";
import { muzoDatabasePath } from "../paths";

export interface ElectronContainerOptions {
  dbPath?: string;
  currentUnixSeconds?: () => number;
  generateLibraryId?: () => string;
  generatePlaylistEntryId?: () => string;
  generatePlaylistId?: () => string;
  generatePlayHistoryId?: () => string;
  generateTrackId?: () => string;
}

export interface ElectronCradle {
  dbPath: string;
  currentUnixSeconds: () => number;
  generateLibraryId: () => string;
  generatePlaylistEntryId: () => string;
  generatePlaylistId: () => string;
  generatePlayHistoryId: () => string;
  generateTrackId: () => string;
  libraries: LibraryRepository;
  playHistory: PlayHistoryRepository;
  playlists: PlaylistRepository;
  tracks: TrackRepository;
  audioSources: AudioSourceRegistry;
  walker: AudioFileWalker;
  reader: AudioMetadataReader;
  playlistService: PlaylistApplicationService;
  prepareTrackAudioSourceService: PrepareTrackAudioSourceApplicationService;
  reconcileFilesystemLibrariesService: ReconcileFilesystemLibrariesService;
  scanLibraryService: ScanLibraryService;
  addLibraryCommand: AddLibraryCommand;
  addTrackToPlaylistCommand: AddTrackToPlaylistCommand;
  createPlaylistCommand: CreatePlaylistCommand;
  listPlaylistsCommand: ListPlaylistsCommand;
  listLibrariesCommand: ListLibrariesCommand;
  listTracksCommand: ListTracksCommand;
  listTrackPlayCountsCommand: ListTrackPlayCountsCommand;
  prepareTrackAudioSourceCommand: PrepareTrackAudioSourceCommand;
  recordTrackPlayCommand: RecordTrackPlayCommand;
  removePlaylistEntryCommand: RemovePlaylistEntryCommand;
  reorderPlaylistEntriesCommand: ReorderPlaylistEntriesCommand;
  scanLibraryCommand: ScanLibraryCommand;
  editTrackMetadataCommand: EditTrackMetadataCommand;
  clearTrackMetadataOverrideCommand: ClearTrackMetadataOverrideCommand;
  commandHandlers: CommandHandler[];
  commandDispatcher: CommandDispatcher;
  migrateDatabase: () => Promise<void>;
  reconcileFilesystemLibraries: () => Promise<{
    librariesReconciled: number;
    failures: Array<{ libraryId: string; message: string }>;
  }>;
}

export function createElectronContainer(
  options: ElectronContainerOptions = {},
): AwilixContainer<ElectronCradle> {
  const container = createContainer<ElectronCradle>({
    injectionMode: InjectionMode.CLASSIC,
  });

  container.register({
    dbPath: asValue(options.dbPath ?? muzoDatabasePath()),
    currentUnixSeconds: asValue(options.currentUnixSeconds ?? currentUnixSeconds),
    generateLibraryId: asValue(options.generateLibraryId ?? ulid),
    generatePlaylistEntryId: asValue(options.generatePlaylistEntryId ?? ulid),
    generatePlaylistId: asValue(options.generatePlaylistId ?? ulid),
    generatePlayHistoryId: asValue(options.generatePlayHistoryId ?? ulid),
    generateTrackId: asValue(options.generateTrackId ?? ulid),
    migrateDatabase: asFunction(
      (dbPath: string) => () => runSqliteMigrations(dbPath),
    ).singleton(),
    libraries: asClass(SqliteLibraryRepository).singleton(),
    playHistory: asClass(SqlitePlayHistoryRepository).singleton(),
    playlists: asClass(SqlitePlaylistRepository).singleton(),
    tracks: asClass(SqliteTrackRepository).singleton(),
    audioSources: asClass(NodeAudioStreamServer).singleton(),
    walker: asClass(NodeAudioFileWalker).singleton(),
    reader: asClass(MusicMetadataReader).singleton(),
    playlistService: asClass(PlaylistApplicationService).singleton(),
    prepareTrackAudioSourceService: asClass(
      PrepareTrackAudioSourceApplicationService,
    ).singleton(),
    reconcileFilesystemLibrariesService: asClass(
      ReconcileFilesystemLibrariesService,
    ).singleton(),
    scanLibraryService: asClass(ScanLibraryService).singleton(),
    addLibraryCommand: asClass(AddLibraryCommand).singleton(),
    addTrackToPlaylistCommand: asClass(AddTrackToPlaylistCommand).singleton(),
    createPlaylistCommand: asClass(CreatePlaylistCommand).singleton(),
    listPlaylistsCommand: asClass(ListPlaylistsCommand).singleton(),
    listLibrariesCommand: asClass(ListLibrariesCommand).singleton(),
    listTracksCommand: asClass(ListTracksCommand).singleton(),
    listTrackPlayCountsCommand: asClass(ListTrackPlayCountsCommand).singleton(),
    prepareTrackAudioSourceCommand: asClass(
      PrepareTrackAudioSourceCommand,
    ).singleton(),
    recordTrackPlayCommand: asClass(RecordTrackPlayCommand).singleton(),
    removePlaylistEntryCommand: asClass(RemovePlaylistEntryCommand).singleton(),
    reorderPlaylistEntriesCommand: asClass(
      ReorderPlaylistEntriesCommand,
    ).singleton(),
    scanLibraryCommand: asClass(ScanLibraryCommand).singleton(),
    editTrackMetadataCommand: asClass(EditTrackMetadataCommand).singleton(),
    clearTrackMetadataOverrideCommand: asClass(
      ClearTrackMetadataOverrideCommand,
    ).singleton(),
    commandHandlers: asFunction(
      (
        addLibraryCommand: AddLibraryCommand,
        addTrackToPlaylistCommand: AddTrackToPlaylistCommand,
        createPlaylistCommand: CreatePlaylistCommand,
        listPlaylistsCommand: ListPlaylistsCommand,
        listLibrariesCommand: ListLibrariesCommand,
        listTracksCommand: ListTracksCommand,
        listTrackPlayCountsCommand: ListTrackPlayCountsCommand,
        prepareTrackAudioSourceCommand: PrepareTrackAudioSourceCommand,
        recordTrackPlayCommand: RecordTrackPlayCommand,
        removePlaylistEntryCommand: RemovePlaylistEntryCommand,
        reorderPlaylistEntriesCommand: ReorderPlaylistEntriesCommand,
        scanLibraryCommand: ScanLibraryCommand,
        editTrackMetadataCommand: EditTrackMetadataCommand,
        clearTrackMetadataOverrideCommand: ClearTrackMetadataOverrideCommand,
      ) => [
        addLibraryCommand,
        addTrackToPlaylistCommand,
        createPlaylistCommand,
        listPlaylistsCommand,
        listLibrariesCommand,
        listTracksCommand,
        listTrackPlayCountsCommand,
        prepareTrackAudioSourceCommand,
        recordTrackPlayCommand,
        removePlaylistEntryCommand,
        reorderPlaylistEntriesCommand,
        scanLibraryCommand,
        editTrackMetadataCommand,
        clearTrackMetadataOverrideCommand,
      ],
    ).singleton(),
    commandDispatcher: asClass(CommandDispatcher).singleton(),
    reconcileFilesystemLibraries: asFunction(
      (reconcileFilesystemLibrariesService: ReconcileFilesystemLibrariesService) =>
        () => reconcileFilesystemLibrariesService.reconcile(),
    ).singleton(),
  });

  return container;
}

function currentUnixSeconds(): number {
  return Math.floor(Date.now() / 1000);
}
