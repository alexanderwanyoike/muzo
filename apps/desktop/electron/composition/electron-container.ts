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
import type { IpcCommandHandler } from "../ipc/ipc-command-handler";
import { LegacyResultCommandAdapter } from "../ipc/legacy-result-command-adapter";
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
import type { Logger } from "../application/interfaces/logger-interfaces";
import type { PlayHistoryRepository } from "../application/interfaces/play-history-interfaces";
import type { PlaylistRepository } from "../application/interfaces/playlist-interfaces";
import type { AudioSourceRegistry } from "../application/interfaces/audio-source-interfaces";
import type {
  AudioFileWalker,
  AudioMetadataReader,
} from "../application/interfaces/scan-interfaces";
import type { FilesystemLibraryWatcher } from "../application/interfaces/watch-interfaces";
import { PrepareTrackAudioSourceApplicationService } from "../application/prepare-track-audio-source-service";
import { PlaylistApplicationService } from "../application/playlist-service";
import { ReconcileFilesystemLibrariesService } from "../application/reconcile-filesystem-libraries-service";
import { NodeAudioStreamServer } from "../infrastructure/audio/node-audio-stream-server";
import { ScanLibraryService } from "../application/scan-library-service";
import { NodeAudioFileWalker } from "../infrastructure/filesystem/node-audio-file-walker";
import { NodeFilesystemLibraryWatcher } from "../infrastructure/filesystem/node-filesystem-library-watcher";
import { WatchFilesystemLibrariesService } from "../application/watch-filesystem-libraries-service";
import { MusicMetadataReader } from "../infrastructure/metadata/music-metadata-reader";
import { ConsoleLogger } from "../infrastructure/logging/console-logger";
import { SqliteLibraryRepository } from "../infrastructure/sqlite/sqlite-library-repository";
import { runSqliteMigrations } from "../infrastructure/sqlite/sqlite-migrations";
import { SqlitePlayHistoryRepository } from "../infrastructure/sqlite/sqlite-play-history-repository";
import { SqlitePlaylistRepository } from "../infrastructure/sqlite/sqlite-playlist-repository";
import { SqliteTrackRepository } from "../infrastructure/sqlite/sqlite-track-repository";
import { CommandDispatcher } from "../ipc/command-dispatcher";
import { muzoDatabasePath } from "../paths";

import { DropboxAccountService } from "../application/dropbox-account-service";
import { DropboxAccountCommand } from "../application/commands/dropbox-account-command";
import type { DropboxTokenCipher } from "../application/interfaces/dropbox-account-interfaces";
import { DropboxApiClient, FetchDropboxHttpTransport, type DropboxHttpTransport } from "../infrastructure/dropbox/dropbox-api-client";
import { SqliteDropboxAccountRepository } from "../infrastructure/sqlite/sqlite-dropbox-account-repository";

import { dirname, join } from "node:path";
import { DropboxLibraryService } from "../application/dropbox-library-service";
import { DropboxTrackAudioSourceService } from "../application/dropbox-track-audio-source-service";
import { ScanDropboxLibraryService } from "../application/scan-dropbox-library-service";
import { DropboxLibraryCommand } from "../application/commands/dropbox-library-command";
import { DropboxLibraryCatalogAdapter } from "../infrastructure/dropbox/dropbox-library-catalog";
import { DropboxLibraryPoller } from "../infrastructure/dropbox/dropbox-library-poller";
import { DropboxAudioDownloader } from "../infrastructure/dropbox/dropbox-audio-downloader";
import { SqliteDropboxLibrarySettings } from "../infrastructure/sqlite/sqlite-dropbox-library-settings";
import type { DropboxLibrarySettings } from "../application/interfaces/dropbox-library-settings";

export interface ElectronContainerOptions {
  dbPath?: string;
  dropboxCipher?: DropboxTokenCipher;
  dropboxDownloadDirectory?: string;
  dropboxDownloadRequest?: typeof fetch;
  dropboxTransport?: DropboxHttpTransport;
  openExternal?: (url: string) => Promise<void>;
  currentUnixSeconds?: () => number;
  generateLibraryId?: () => string;
  generatePlaylistEntryId?: () => string;
  generatePlaylistId?: () => string;
  generatePlayHistoryId?: () => string;
  generateTrackId?: () => string;
}

export interface ElectronCradle {
  dbPath: string;
  dropboxAccountService: DropboxAccountService;
  dropboxLibraries: DropboxLibraryService;
  dropboxSettings: DropboxLibrarySettings;
  dropboxPoller: DropboxLibraryPoller;
  dropboxAudioSources: DropboxTrackAudioSourceService;
  currentUnixSeconds: () => number;
  generateLibraryId: () => string;
  generatePlaylistEntryId: () => string;
  generatePlaylistId: () => string;
  generatePlayHistoryId: () => string;
  generateTrackId: () => string;
  logger: Logger;
  libraries: LibraryRepository;
  playHistory: PlayHistoryRepository;
  playlists: PlaylistRepository;
  tracks: TrackRepository;
  audioSources: AudioSourceRegistry;
  walker: AudioFileWalker;
  filesystemWatcher: FilesystemLibraryWatcher;
  reader: AudioMetadataReader;
  playlistService: PlaylistApplicationService;
  prepareTrackAudioSourceService: PrepareTrackAudioSourceApplicationService;
  reconcileFilesystemLibrariesService: ReconcileFilesystemLibrariesService;
  scanLibraryService: ScanLibraryService;
  watchFilesystemLibrariesService: WatchFilesystemLibrariesService;
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
  commandHandlers: IpcCommandHandler[];
  commandDispatcher: CommandDispatcher;
  migrateDatabase: () => Promise<void>;
  reconcileFilesystemLibraries: () => Promise<{
    librariesReconciled: number;
    failures: Array<{ libraryId: string; message: string }>;
  }>;
  watchFilesystemLibraries: () => Promise<{
    librariesWatched: number;
    failures: Array<{ libraryId: string; message: string }>;
  }>;
}

export function createElectronContainer(
  options: ElectronContainerOptions = {},
): AwilixContainer<ElectronCradle> {
  const container = createContainer<ElectronCradle>({
    injectionMode: InjectionMode.PROXY,
  });

  const dropboxTransport = options.dropboxTransport ?? new FetchDropboxHttpTransport();
  container.register({
    dropboxSettings: asFunction(({ dbPath }: ElectronCradle) => new SqliteDropboxLibrarySettings(dbPath)).singleton(),
    dropboxLibraries: asFunction(({ libraries, tracks, dropboxAccountService, dropboxSettings, generateLibraryId, generateTrackId }: ElectronCradle) =>
      new DropboxLibraryService(libraries, dropboxAccountService, dropboxSettings,
        new ScanDropboxLibraryService(libraries, tracks,
          new DropboxLibraryCatalogAdapter(new DropboxApiClient({ clientId: "" }, dropboxTransport)), generateTrackId), generateLibraryId),
    ).singleton(),
    dropboxPoller: asFunction(({ dropboxLibraries, dropboxSettings, currentUnixSeconds }: ElectronCradle) =>
      new DropboxLibraryPoller(dropboxLibraries, dropboxSettings, currentUnixSeconds),
    ).singleton(),
    dropboxAudioSources: asFunction(({ dropboxLibraries, tracks, audioSources, dbPath }: ElectronCradle) =>
      new DropboxTrackAudioSourceService(dropboxLibraries, tracks,
        new DropboxAudioDownloader(options.dropboxDownloadDirectory ?? join(dirname(dbPath), "dropbox-audio"), options.dropboxDownloadRequest), audioSources),
    ).singleton(),
    dropboxAccountService: asFunction(({ dbPath, currentUnixSeconds }: ElectronCradle) =>
      new DropboxAccountService(
        new SqliteDropboxAccountRepository(dbPath, options.dropboxCipher ?? {
          protect: () => { throw new Error("Secure storage is unavailable"); },
          unprotect: () => { throw new Error("Secure storage is unavailable"); },
        }),
        (clientId) => new DropboxApiClient({ clientId }, dropboxTransport),
        options.openExternal ?? (async () => { throw new Error("System browser is unavailable"); }),
        currentUnixSeconds,
      ),
    ).singleton(),
    dbPath: asValue(options.dbPath ?? muzoDatabasePath()),
    currentUnixSeconds: asValue(options.currentUnixSeconds ?? currentUnixSeconds),
    generateLibraryId: asValue(options.generateLibraryId ?? ulid),
    generatePlaylistEntryId: asValue(options.generatePlaylistEntryId ?? ulid),
    generatePlaylistId: asValue(options.generatePlaylistId ?? ulid),
    generatePlayHistoryId: asValue(options.generatePlayHistoryId ?? ulid),
    generateTrackId: asValue(options.generateTrackId ?? ulid),
    logger: asClass(ConsoleLogger).singleton(),
    migrateDatabase: asFunction(
      ({ dbPath }: ElectronCradle) => () => runSqliteMigrations(dbPath),
    ).singleton(),
    libraries: asFunction(
      ({ dbPath }: ElectronCradle) => new SqliteLibraryRepository(dbPath),
    ).singleton(),
    playHistory: asFunction(
      ({ dbPath }: ElectronCradle) => new SqlitePlayHistoryRepository(dbPath),
    ).singleton(),
    playlists: asFunction(
      ({ dbPath }: ElectronCradle) => new SqlitePlaylistRepository(dbPath),
    ).singleton(),
    tracks: asFunction(
      ({ dbPath }: ElectronCradle) => new SqliteTrackRepository(dbPath),
    ).singleton(),
    audioSources: asClass(NodeAudioStreamServer).singleton(),
    walker: asClass(NodeAudioFileWalker).singleton(),
    filesystemWatcher: asClass(NodeFilesystemLibraryWatcher).singleton(),
    reader: asClass(MusicMetadataReader).singleton(),
    playlistService: asFunction(
      ({ playlists, generatePlaylistId, generatePlaylistEntryId }: ElectronCradle) =>
        new PlaylistApplicationService(
          playlists,
          generatePlaylistId,
          generatePlaylistEntryId,
        ),
    ).singleton(),
    prepareTrackAudioSourceService: asFunction(
      ({ tracks, audioSources }: ElectronCradle) =>
        new PrepareTrackAudioSourceApplicationService(tracks, audioSources),
    ).singleton(),
    reconcileFilesystemLibrariesService: asFunction(
      ({ libraries, scanLibraryService }: ElectronCradle) =>
        new ReconcileFilesystemLibrariesService(libraries, scanLibraryService),
    ).singleton(),
    scanLibraryService: asFunction(
      ({ libraries, tracks, walker, reader, generateTrackId }: ElectronCradle) =>
        new ScanLibraryService(
          libraries,
          tracks,
          walker,
          reader,
          generateTrackId,
        ),
    ).singleton(),
    watchFilesystemLibrariesService: asFunction(
      ({
        libraries,
        filesystemWatcher,
        scanLibraryService,
        logger,
      }: ElectronCradle) =>
        new WatchFilesystemLibrariesService(
          libraries,
          filesystemWatcher,
          scanLibraryService,
          logger,
        ),
    ).singleton(),
    addLibraryCommand: asFunction(
      ({
        libraries,
        generateLibraryId,
        watchFilesystemLibrariesService,
        scanLibraryService,
        logger,
      }: ElectronCradle) =>
        new AddLibraryCommand(
          libraries,
          generateLibraryId,
          watchFilesystemLibrariesService,
          scanLibraryService,
          logger,
        ),
    ).singleton(),
    addTrackToPlaylistCommand: asFunction(
      ({ playlistService }: ElectronCradle) =>
        new AddTrackToPlaylistCommand(playlistService),
    ).singleton(),
    createPlaylistCommand: asFunction(
      ({ playlistService }: ElectronCradle) =>
        new CreatePlaylistCommand(playlistService),
    ).singleton(),
    listPlaylistsCommand: asFunction(
      ({ playlistService }: ElectronCradle) =>
        new ListPlaylistsCommand(playlistService),
    ).singleton(),
    listLibrariesCommand: asFunction(
      ({ libraries }: ElectronCradle) => new ListLibrariesCommand(libraries),
    ).singleton(),
    listTracksCommand: asFunction(
      ({ tracks }: ElectronCradle) => new ListTracksCommand(tracks),
    ).singleton(),
    listTrackPlayCountsCommand: asFunction(
      ({ playHistory }: ElectronCradle) =>
        new ListTrackPlayCountsCommand(playHistory),
    ).singleton(),
    prepareTrackAudioSourceCommand: asFunction(
      ({ prepareTrackAudioSourceService, libraries, dropboxAudioSources }: ElectronCradle) =>
        new PrepareTrackAudioSourceCommand(prepareTrackAudioSourceService, libraries, dropboxAudioSources),
    ).singleton(),
    recordTrackPlayCommand: asFunction(
      ({
        playHistory,
        generatePlayHistoryId,
        currentUnixSeconds,
      }: ElectronCradle) =>
        new RecordTrackPlayCommand(
          playHistory,
          generatePlayHistoryId,
          currentUnixSeconds,
        ),
    ).singleton(),
    removePlaylistEntryCommand: asFunction(
      ({ playlistService }: ElectronCradle) =>
        new RemovePlaylistEntryCommand(playlistService),
    ).singleton(),
    reorderPlaylistEntriesCommand: asFunction(
      ({ playlistService }: ElectronCradle) =>
        new ReorderPlaylistEntriesCommand(playlistService),
    ).singleton(),
    scanLibraryCommand: asFunction(
      ({ scanLibraryService }: ElectronCradle) =>
        new ScanLibraryCommand(scanLibraryService),
    ).singleton(),
    editTrackMetadataCommand: asFunction(
      ({ tracks }: ElectronCradle) => new EditTrackMetadataCommand(tracks),
    ).singleton(),
    clearTrackMetadataOverrideCommand: asFunction(
      ({ tracks }: ElectronCradle) =>
        new ClearTrackMetadataOverrideCommand(tracks),
    ).singleton(),
    commandHandlers: asFunction((cradle: ElectronCradle) => [
      ...(["dropbox_begin", "dropbox_complete", "dropbox_status", "dropbox_disconnect"] as const)
        .map((command) => new DropboxAccountCommand(command, cradle.dropboxAccountService)),
      ...(["dropbox_add_library", "dropbox_sync_library", "dropbox_sync_status", "dropbox_set_sync_interval"] as const)
        .map((command) => new DropboxLibraryCommand(command, cradle.dropboxLibraries, cradle.dropboxPoller)),
      new LegacyResultCommandAdapter(cradle.addLibraryCommand),
      new LegacyResultCommandAdapter(cradle.addTrackToPlaylistCommand),
      new LegacyResultCommandAdapter(cradle.createPlaylistCommand),
      new LegacyResultCommandAdapter(cradle.listPlaylistsCommand),
      new LegacyResultCommandAdapter(cradle.listLibrariesCommand),
      new LegacyResultCommandAdapter(cradle.listTracksCommand),
      new LegacyResultCommandAdapter(cradle.listTrackPlayCountsCommand),
      new LegacyResultCommandAdapter(cradle.prepareTrackAudioSourceCommand),
      new LegacyResultCommandAdapter(cradle.recordTrackPlayCommand),
      new LegacyResultCommandAdapter(cradle.removePlaylistEntryCommand),
      new LegacyResultCommandAdapter(cradle.reorderPlaylistEntriesCommand),
      new LegacyResultCommandAdapter(cradle.scanLibraryCommand),
      new LegacyResultCommandAdapter(cradle.editTrackMetadataCommand),
      new LegacyResultCommandAdapter(cradle.clearTrackMetadataOverrideCommand),
    ]).singleton(),
    commandDispatcher: asFunction(
      ({ commandHandlers }: ElectronCradle) =>
        new CommandDispatcher(commandHandlers),
    ).singleton(),
    reconcileFilesystemLibraries: asFunction(
      ({ reconcileFilesystemLibrariesService }: ElectronCradle) => () =>
        reconcileFilesystemLibrariesService.reconcile(),
    ).singleton(),
    watchFilesystemLibraries: asFunction(
      ({ watchFilesystemLibrariesService }: ElectronCradle) => () =>
        watchFilesystemLibrariesService.watch(),
    ).singleton(),
  });

  return container;
}

function currentUnixSeconds(): number {
  return Math.floor(Date.now() / 1000);
}
