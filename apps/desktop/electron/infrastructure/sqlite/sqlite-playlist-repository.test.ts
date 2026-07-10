// @vitest-environment node

import { join } from "node:path";
import { tmpdir } from "node:os";

import { describe, expect, it } from "vitest";

import { runSqliteMigrations } from "./sqlite-migrations";
import { SqlitePlaylistRepository } from "./sqlite-playlist-repository";

describe("Electron playlist repository", () => {
  it("persists playlists with entries in order", async () => {
    const dbPath = tempDatabasePath();
    await runSqliteMigrations(dbPath);
    const repository = new SqlitePlaylistRepository(dbPath);

    await repository.add({
      id: "playlist-1",
      name: "Road Trip",
      entries: [],
    });
    await repository.save({
      id: "playlist-1",
      name: "Road Trip",
      entries: [
        { id: "entry-2", trackId: "track-2", position: 0 },
        { id: "entry-1", trackId: "track-1", position: 1 },
      ],
    });

    await expect(
      new SqlitePlaylistRepository(dbPath).findById("playlist-1"),
    ).resolves.toEqual({
      id: "playlist-1",
      name: "Road Trip",
      entries: [
        {
          id: "entry-2",
          trackId: "track-2",
          trackTitle: null,
          trackArtist: null,
          trackLibraryId: null,
          trackDurationSeconds: null,
          position: 0,
        },
        {
          id: "entry-1",
          trackId: "track-1",
          trackTitle: null,
          trackArtist: null,
          trackLibraryId: null,
          trackDurationSeconds: null,
          position: 1,
        },
      ],
    });
    await expect(new SqlitePlaylistRepository(dbPath).list()).resolves.toEqual([
      {
        id: "playlist-1",
        name: "Road Trip",
        entries: [
          {
            id: "entry-2",
            trackId: "track-2",
            trackTitle: null,
            trackArtist: null,
            trackLibraryId: null,
            trackDurationSeconds: null,
            position: 0,
          },
          {
            id: "entry-1",
            trackId: "track-1",
            trackTitle: null,
            trackArtist: null,
            trackLibraryId: null,
            trackDurationSeconds: null,
            position: 1,
          },
        ],
      },
    ]);
  });

  it("lists playlist entries with track labels when the track still exists", async () => {
    const dbPath = tempDatabasePath();
    await runSqliteMigrations(dbPath);
    const repository = new SqlitePlaylistRepository(dbPath);

    await insertTrack(dbPath, {
      id: "track-1",
      title: "Hotel California",
      artist: "Eagles",
      filePath: "/music/hotel-california.mp3",
    });
    await repository.add({
      id: "playlist-1",
      name: "Road Trip",
      entries: [],
    });
    await repository.save({
      id: "playlist-1",
      name: "Road Trip",
      entries: [
        { id: "entry-1", trackId: "track-1", position: 0 },
        { id: "entry-2", trackId: "missing-track", position: 1 },
      ],
    });

    await expect(repository.list()).resolves.toEqual([
      {
        id: "playlist-1",
        name: "Road Trip",
        entries: [
          {
            id: "entry-1",
            trackId: "track-1",
            trackTitle: "Hotel California",
            trackArtist: "Eagles",
            trackLibraryId: "lib-1",
            trackDurationSeconds: 391,
            position: 0,
          },
          {
            id: "entry-2",
            trackId: "missing-track",
            trackTitle: null,
            trackArtist: null,
            trackLibraryId: null,
            trackDurationSeconds: null,
            position: 1,
          },
        ],
      },
    ]);
  });
});

function tempDatabasePath(): string {
  return join(tmpdir(), `muzo-${Date.now()}-${Math.random()}.sqlite`);
}

async function insertTrack(
  dbPath: string,
  input: {
    artist: string;
    filePath: string;
    id: string;
    title: string;
  },
): Promise<void> {
  const { default: initSqlJs } = await import("sql.js");
  const SQL = await initSqlJs();
  const { readFileSync, writeFileSync } = await import("node:fs");
  const database = new SQL.Database(readFileSync(dbPath));
  try {
    database.run(
      `
        INSERT INTO tracks (
          id, library_id, title, artist, duration_seconds,
          file_path, file_size, file_mtime
        ) VALUES (?, ?, ?, ?, ?, ?, ?, ?)
      `,
      [
        input.id,
        "lib-1",
        input.title,
        input.artist,
        391,
        input.filePath,
        1000,
        1,
      ],
    );
    writeFileSync(dbPath, database.export());
  } finally {
    database.close();
  }
}
