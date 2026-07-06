import { existsSync, readFileSync } from "node:fs";
import { createRequire } from "node:module";

import initSqlJs, { type SqlJsStatic } from "sql.js";

import type { TrackDto } from "../../../src/api";
import type { TrackRepository } from "../../application/interfaces/repository-interfaces";

let sqlModulePromise: Promise<SqlJsStatic> | null = null;

export class SqliteTrackRepository implements TrackRepository {
  constructor(private readonly dbPath: string) {}

  async listForLibrary(libraryId: string): Promise<TrackDto[]> {
    if (!existsSync(this.dbPath)) {
      return [];
    }

    const SQL = await loadSqlModule();
    const database = new SQL.Database(readFileSync(this.dbPath));
    try {
      if (!tableExists(database, "tracks")) {
        return [];
      }

      const statement = database.prepare(`
        SELECT
          id, library_id, title, artist, album, track_number, disc_number, genre, year,
          override_title, override_artist, override_album, override_track_number,
          override_disc_number, override_genre, override_year,
          duration_seconds, file_path
        FROM tracks WHERE library_id = ? ORDER BY rowid
      `);
      try {
        statement.bind([libraryId]);
        const tracks: TrackDto[] = [];
        while (statement.step()) {
          tracks.push(trackFromRow(statement.getAsObject()));
        }
        return tracks;
      } finally {
        statement.free();
      }
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

function tableExists(
  database: InstanceType<SqlJsStatic["Database"]>,
  tableName: string,
): boolean {
  const statement = database.prepare(
    "SELECT name FROM sqlite_master WHERE type = 'table' AND name = ?",
  );
  try {
    statement.bind([tableName]);
    return statement.step();
  } finally {
    statement.free();
  }
}

function trackFromRow(row: Record<string, unknown>): TrackDto {
  return {
    id: stringColumn(row, "id"),
    libraryId: stringColumn(row, "library_id"),
    title: overrideString(row, "override_title") ?? stringColumn(row, "title"),
    artist: overrideString(row, "override_artist") ?? stringColumn(row, "artist"),
    album: overrideString(row, "override_album") ?? nullableString(row, "album"),
    trackNumber:
      overrideNumber(row, "override_track_number") ??
      nullableNumber(row, "track_number"),
    discNumber:
      overrideNumber(row, "override_disc_number") ??
      nullableNumber(row, "disc_number"),
    genre: overrideString(row, "override_genre") ?? nullableString(row, "genre"),
    year: overrideNumber(row, "override_year") ?? nullableNumber(row, "year"),
    metadataOverridden: metadataOverridden(row),
    durationSeconds: numberColumn(row, "duration_seconds"),
    filePath: stringColumn(row, "file_path"),
  };
}

function metadataOverridden(row: Record<string, unknown>): boolean {
  return [
    "override_title",
    "override_artist",
    "override_album",
    "override_track_number",
    "override_disc_number",
    "override_genre",
    "override_year",
  ].some((column) => row[column] !== null && row[column] !== undefined);
}

function overrideString(
  row: Record<string, unknown>,
  column: string,
): string | null {
  return nullableString(row, column);
}

function overrideNumber(
  row: Record<string, unknown>,
  column: string,
): number | null {
  return nullableNumber(row, column);
}

function nullableString(
  row: Record<string, unknown>,
  column: string,
): string | null {
  const value = row[column];
  if (value === null || value === undefined) {
    return null;
  }
  if (typeof value !== "string") {
    throw new Error(`expected ${column} to be a string`);
  }
  return value;
}

function stringColumn(row: Record<string, unknown>, column: string): string {
  const value = nullableString(row, column);
  if (value === null) {
    throw new Error(`expected ${column} to be present`);
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
