import { readFileSync, writeFileSync } from "node:fs";
import { createRequire } from "node:module";

import initSqlJs, { type SqlJsStatic } from "sql.js";

import type {
  PlayCount,
  PlayHistoryEntry,
  PlayHistoryRepository,
} from "../../application/interfaces/play-history-interfaces";

let sqlModulePromise: Promise<SqlJsStatic> | null = null;

export class SqlitePlayHistoryRepository implements PlayHistoryRepository {
  constructor(private readonly dbPath: string) {}

  async listPlayCounts(libraryId: string): Promise<PlayCount[]> {
    const SQL = await loadSqlModule();
    const database = new SQL.Database(readFileSync(this.dbPath));
    try {
      const statement = database.prepare(`
        SELECT track_id, COUNT(*) AS play_count, MAX(played_at_unix_seconds) AS last_played_at_unix_seconds
        FROM play_history
        WHERE library_id = ?
        GROUP BY track_id
        ORDER BY MAX(played_at_unix_seconds) DESC, track_id ASC
      `);
      try {
        statement.bind([libraryId]);
        const counts: PlayCount[] = [];
        while (statement.step()) {
          counts.push(playCountFromRow(statement.getAsObject()));
        }
        return counts;
      } finally {
        statement.free();
      }
    } finally {
      database.close();
    }
  }

  async recordPlay(entry: PlayHistoryEntry): Promise<void> {
    const SQL = await loadSqlModule();
    const database = new SQL.Database(readFileSync(this.dbPath));
    try {
      database.run(
        `INSERT INTO play_history (
          id, library_id, track_id, played_at_unix_seconds
        ) VALUES (?, ?, ?, ?)`,
        [
          entry.id,
          entry.libraryId,
          entry.trackId,
          entry.playedAtUnixSeconds,
        ],
      );
      writeFileSync(this.dbPath, database.export());
    } finally {
      database.close();
    }
  }
}

function loadSqlModule(): Promise<SqlJsStatic> {
  sqlModulePromise ??= initSqlJs({
    locateFile: (file) => {
      const require = createRequire(import.meta.url);
      return require.resolve(`sql.js/dist/${file}`);
    },
  });
  return sqlModulePromise;
}

function playCountFromRow(row: Record<string, unknown>): PlayCount {
  return {
    trackId: stringColumn(row, "track_id"),
    playCount: numberColumn(row, "play_count"),
    lastPlayedAtUnixSeconds: nullableNumber(
      row,
      "last_played_at_unix_seconds",
    ),
  };
}

function stringColumn(row: Record<string, unknown>, column: string): string {
  const value = row[column];
  if (typeof value !== "string") {
    throw new Error(`expected ${column} to be a string`);
  }
  return value;
}

function nullableNumber(
  row: Record<string, unknown>,
  column: string,
): number | null {
  const value = row[column];
  if (value === null || value === undefined) {
    return null;
  }
  if (typeof value !== "number") {
    throw new Error(`expected ${column} to be a number`);
  }
  return value;
}

function numberColumn(row: Record<string, unknown>, column: string): number {
  const value = nullableNumber(row, column);
  if (value === null) {
    throw new Error(`expected ${column} to be present`);
  }
  return value;
}
