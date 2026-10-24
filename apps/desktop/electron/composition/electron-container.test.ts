// @vitest-environment node

import { join } from "node:path";
import { tmpdir } from "node:os";
import { mkdirSync, rmSync } from "node:fs";

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
        entries: [
          {
            id: "entry-1",
            trackId: "trk-1",
            trackTitle: "File Title",
            trackArtist: "File Artist",
            trackLibraryId: "lib-1",
            trackDurationSeconds: 120,
            position: 0,
          },
        ],
      },
    ]);
  });
  it("connects Dropbox through IPC and exposes only public account status", async () => {
    const opened: string[] = [];
    const container = createElectronContainer({
      dbPath: tempDatabasePath(),
      openExternal: async (url) => { opened.push(url); },
      dropboxCipher: { protect: (s) => `protected:${s}`, unprotect: (s) => s.slice(10) },
      dropboxTransport: { send: async () => ({ status: 200, body: JSON.stringify({ access_token: "access", refresh_token: "refresh", account_id: "dbid:1", expires_in: 3600 }) }) },
    });
    await container.resolve("migrateDatabase")();
    const commands = container.resolve("commandDispatcher");
    expect(await commands.handleElectronCommand("dropbox_begin", { input: { clientId: "app-key" } })).toEqual({ ok: true, value: { pending: true } });
    expect(opened[0]).toContain("https://www.dropbox.com/oauth2/authorize?");
    expect(await commands.handleElectronCommand("dropbox_complete", { input: { code: "auth-code" } })).toEqual({ ok: true, value: { connected: true, accountId: "dbid:1" } });
    expect(await commands.handleElectronCommand("dropbox_status")).toEqual({ ok: true, value: { connected: true, accountId: "dbid:1" } });
    expect(await commands.handleElectronCommand("dropbox_begin", { input: { clientId: 42 } })).toMatchObject({ ok: false, error: { kind: "invalidInput" } });
    expect(await commands.handleElectronCommand("dropbox_disconnect")).toEqual({ ok: true, value: { connected: false, accountId: null } });
  });

  it("adds, syncs, and plays a Dropbox library through the real SQLite and local audio adapters", async () => {
    const root = tempDirectory();
    const downloadRequest = async () => new Response(new Uint8Array([1, 2, 3, 4]));
    const container = createElectronContainer({
      dbPath: join(root, "muzo.sqlite"), dropboxDownloadDirectory: join(root, "audio"), dropboxDownloadRequest: downloadRequest,
      generateLibraryId: () => "cloud-1", generateTrackId: () => "song-1",
      dropboxCipher: { protect: (value) => value, unprotect: (value) => value }, openExternal: async () => {},
      dropboxTransport: { send: async (request) => ({ status: 200, body: JSON.stringify(request.url.endsWith("/token")
        ? { access_token: "token", refresh_token: "refresh", account_id: "account-1", expires_in: 3600 }
        : { entries: [{ ".tag": "file", name: "Song.mp3", id: "remote-1", path_display: "/Music/Song.mp3", size: 4, server_modified: "2026-10-24T00:00:00Z" }], cursor: "cursor", has_more: false }) }) },
    });
    try {
      await container.resolve("migrateDatabase")();
      const command = container.resolve("commandDispatcher");
      await command.handleElectronCommand("dropbox_begin", { input: { clientId: "app-key" } });
      await command.handleElectronCommand("dropbox_complete", { input: { code: "code" } });
      expect(await command.handleElectronCommand("dropbox_add_library", { input: { name: "Cloud", location: "/Music" } })).toEqual({ ok: true, value: { id: "cloud-1", name: "Cloud", location: "/Music", kind: "dropbox" } });
      expect(await command.handleElectronCommand("dropbox_sync_library", { input: { libraryId: "cloud-1" } })).toEqual({ ok: true, value: { tracksScanned: 1 } });
      const source = await command.handleElectronCommand("prepare_track_audio_source", { input: { libraryId: "cloud-1", trackId: "song-1" } }) as { url: string; mimeType: string };
      expect(source.mimeType).toBe("audio/mpeg");
      expect(Buffer.from(await (await fetch(source.url)).arrayBuffer())).toEqual(Buffer.from([1, 2, 3, 4]));
      expect(await command.handleElectronCommand("dropbox_set_sync_interval", { input: { minutes: 15 } })).toEqual({ ok: true, value: { intervalMinutes: 15 } });
      expect(await command.handleElectronCommand("dropbox_sync_status")).toMatchObject({ ok: true, value: { intervalMinutes: 15 } });
      await command.handleElectronCommand("dropbox_disconnect");
      expect(await command.handleElectronCommand("dropbox_sync_library", { input: { libraryId: "cloud-1" } })).toMatchObject({ ok: false, error: { kind: "notConnected" } });
    } finally {
      if (container.hasRegistration("dropboxPoller")) container.resolve("dropboxPoller").stop();
      rmSync(root, { recursive: true, force: true });
    }
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
  return () => { next += 1; return `entry-${next}`; };
}
