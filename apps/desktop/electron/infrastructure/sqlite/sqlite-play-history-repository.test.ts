// @vitest-environment node

import { join } from "node:path";
import { tmpdir } from "node:os";

import { describe, expect, it } from "vitest";

import { runSqliteMigrations } from "./sqlite-migrations";
import { SqlitePlayHistoryRepository } from "./sqlite-play-history-repository";

describe("Electron play history repository", () => {
  it("records plays and lists counts for a library", async () => {
    const dbPath = tempDatabasePath();
    await runSqliteMigrations(dbPath);
    const repository = new SqlitePlayHistoryRepository(dbPath);

    await repository.recordPlay({
      id: "play-1",
      libraryId: "lib-1",
      trackId: "trk-1",
      playedAtUnixSeconds: 1_719_000_000,
    });
    await repository.recordPlay({
      id: "play-2",
      libraryId: "lib-1",
      trackId: "trk-2",
      playedAtUnixSeconds: 1_719_000_200,
    });
    await repository.recordPlay({
      id: "play-3",
      libraryId: "lib-1",
      trackId: "trk-1",
      playedAtUnixSeconds: 1_719_000_100,
    });
    await repository.recordPlay({
      id: "play-4",
      libraryId: "lib-2",
      trackId: "trk-1",
      playedAtUnixSeconds: 1_719_000_300,
    });

    await expect(repository.listPlayCounts("lib-1")).resolves.toEqual([
      {
        trackId: "trk-2",
        playCount: 1,
        lastPlayedAtUnixSeconds: 1_719_000_200,
      },
      {
        trackId: "trk-1",
        playCount: 2,
        lastPlayedAtUnixSeconds: 1_719_000_100,
      },
    ]);
  });
});

function tempDatabasePath(): string {
  return join(tmpdir(), `muzo-${Date.now()}-${Math.random()}.sqlite`);
}
