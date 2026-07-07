// @vitest-environment node

import { join } from "node:path";
import { tmpdir } from "node:os";
import { mkdirSync } from "node:fs";

import { describe, expect, it } from "vitest";

import { createElectronContainer } from "./electron-container";

describe("Electron container", () => {
  it("wires command handlers with infrastructure dependencies", async () => {
    const libraryRoot = tempDirectory();
    const container = createElectronContainer({
      dbPath: tempDatabasePath(),
      generateLibraryId: () => "lib-1",
      generatePlaylistEntryId: playlistEntryIdGenerator(),
      generatePlaylistId: () => "playlist-1",
    });
    const migrateDatabase = container.resolve("migrateDatabase");
    const reconcileFilesystemLibraries = container.resolve(
      "reconcileFilesystemLibraries",
    );
    const watchFilesystemLibraries = container.resolve("watchFilesystemLibraries");
    const commandDispatcher = container.resolve("commandDispatcher");
    const tracks = container.resolve("tracks");

    await migrateDatabase();

    await expect(
      commandDispatcher.handleElectronCommand("add_library", {
        input: {
          name: "Local Music",
          kind: "filesystem",
          location: libraryRoot,
        },
      }),
    ).resolves.toEqual({
      id: "lib-1",
      name: "Local Music",
      kind: "filesystem",
      location: libraryRoot,
    });

    await expect(
      commandDispatcher.handleElectronCommand("scan_library", {
        input: { libraryId: "lib-1" },
      }),
    ).resolves.toEqual({ tracksScanned: 0 });

    await expect(
      commandDispatcher.handleElectronCommand("list_libraries"),
    ).resolves.toEqual([
      {
        id: "lib-1",
        name: "Local Music",
        kind: "filesystem",
        location: libraryRoot,
      },
    ]);

    await expect(reconcileFilesystemLibraries()).resolves.toEqual({
      librariesReconciled: 1,
      failures: [],
    });

    await expect(watchFilesystemLibraries()).resolves.toEqual({
      librariesWatched: 1,
      failures: [],
    });

    await tracks.upsertScannedTrack({
      id: "trk-1",
      libraryId: "lib-1",
      title: "File Title",
      artist: "File Artist",
      album: "File Album",
      trackNumber: 1,
      discNumber: null,
      genre: null,
      year: null,
      durationSeconds: 120,
      filePath: join(libraryRoot, "song.mp3"),
      fileSize: 1000,
      fileMtime: 123,
    });

    await expect(
      commandDispatcher.handleElectronCommand("edit_track_metadata", {
        input: {
          libraryId: "lib-1",
          trackId: "trk-1",
          title: "Edited Title",
          artist: null,
          album: null,
          trackNumber: null,
          discNumber: null,
          genre: null,
          year: null,
        },
      }),
    ).resolves.toBeUndefined();

    await expect(
      commandDispatcher.handleElectronCommand("list_tracks", {
        libraryId: "lib-1",
      }),
    ).resolves.toEqual([
      {
        id: "trk-1",
        libraryId: "lib-1",
        title: "Edited Title",
        artist: "File Artist",
        album: "File Album",
        trackNumber: 1,
        discNumber: null,
        genre: null,
        year: null,
        metadataOverridden: true,
        durationSeconds: 120,
        filePath: join(libraryRoot, "song.mp3"),
      },
    ]);

    await expect(
      commandDispatcher.handleElectronCommand("clear_track_metadata_override", {
        input: { libraryId: "lib-1", trackId: "trk-1" },
      }),
    ).resolves.toBeUndefined();

    await expect(
      commandDispatcher.handleElectronCommand("list_tracks", {
        libraryId: "lib-1",
      }),
    ).resolves.toEqual([
      {
        id: "trk-1",
        libraryId: "lib-1",
        title: "File Title",
        artist: "File Artist",
        album: "File Album",
        trackNumber: 1,
        discNumber: null,
        genre: null,
        year: null,
        metadataOverridden: false,
        durationSeconds: 120,
        filePath: join(libraryRoot, "song.mp3"),
      },
    ]);

    await expect(
      commandDispatcher.handleElectronCommand("record_track_play", {
        input: { libraryId: "lib-1", trackId: "trk-1" },
      }),
    ).resolves.toBeUndefined();

    await expect(
      commandDispatcher.handleElectronCommand("list_track_play_counts", {
        input: { libraryId: "lib-1" },
      }),
    ).resolves.toEqual([
      {
        trackId: "trk-1",
        playCount: 1,
        lastPlayedAtUnixSeconds: expect.any(Number),
      },
    ]);

    await expect(
      commandDispatcher.handleElectronCommand("prepare_track_audio_source", {
        input: { libraryId: "lib-1", trackId: "trk-1" },
      }),
    ).resolves.toMatchObject({
      mimeType: "audio/mpeg",
      url: expect.stringMatching(/^http:\/\/127\.0\.0\.1:\d+\/audio\/.+/),
    });

    await expect(
      commandDispatcher.handleElectronCommand("create_playlist", {
        input: { name: "Road Trip" },
      }),
    ).resolves.toEqual({
      id: "playlist-1",
      name: "Road Trip",
      entries: [],
    });

    await expect(
      commandDispatcher.handleElectronCommand("add_track_to_playlist", {
        input: { playlistId: "playlist-1", trackId: "trk-1" },
      }),
    ).resolves.toEqual({
      id: "playlist-1",
      name: "Road Trip",
      entries: [{ id: "entry-1", trackId: "trk-1", position: 0 }],
    });

    await expect(
      commandDispatcher.handleElectronCommand("list_playlists"),
    ).resolves.toEqual([
      {
        id: "playlist-1",
        name: "Road Trip",
        entries: [{ id: "entry-1", trackId: "trk-1", position: 0 }],
      },
    ]);
  });
});

function tempDatabasePath(): string {
  return join(tmpdir(), `muzo-${Date.now()}-${Math.random()}.sqlite`);
}

function tempDirectory(): string {
  const directory = join(tmpdir(), `muzo-${Date.now()}-${Math.random()}`);
  mkdirSync(directory, { recursive: true });
  return directory;
}

function playlistEntryIdGenerator(): () => string {
  let next = 0;
  return () => `entry-${++next}`;
}
