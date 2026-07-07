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
        { id: "entry-2", trackId: "track-2", position: 0 },
        { id: "entry-1", trackId: "track-1", position: 1 },
      ],
    });
    await expect(new SqlitePlaylistRepository(dbPath).list()).resolves.toEqual([
      {
        id: "playlist-1",
        name: "Road Trip",
        entries: [
          { id: "entry-2", trackId: "track-2", position: 0 },
          { id: "entry-1", trackId: "track-1", position: 1 },
        ],
      },
    ]);
  });
});

function tempDatabasePath(): string {
  return join(tmpdir(), `muzo-${Date.now()}-${Math.random()}.sqlite`);
}
