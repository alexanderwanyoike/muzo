// @vitest-environment node

import { mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { join } from "node:path";
import { tmpdir } from "node:os";

import initSqlJs, { type Database } from "sql.js";
import { describe, expect, it } from "vitest";

import { runSqliteMigrations } from "./sqlite-migrations";

describe("Electron SQLite migrations", () => {
  it("migrates a fresh database to the current schema", async () => {
    const dbPath = tempDatabasePath();

    await runSqliteMigrations(dbPath);

    const database = await openDatabase(dbPath);
    try {
      expect(userVersion(database)).toBe(5);
      expect(tableColumns(database, "libraries")).toEqual([
        "id",
        "name",
        "kind",
        "location",
      ]);
      expect(tableColumns(database, "tracks")).toEqual([
        "id",
        "library_id",
        "title",
        "artist",
        "duration_seconds",
        "file_path",
        "file_size",
        "file_mtime",
        "album",
        "track_number",
        "disc_number",
        "genre",
        "year",
        "override_title",
        "override_artist",
        "override_album",
        "override_track_number",
        "override_disc_number",
        "override_genre",
        "override_year",
      ]);
      expect(tableColumns(database, "playlists")).toEqual(["id", "name"]);
      expect(tableColumns(database, "playlist_entries")).toEqual([
        "id",
        "playlist_id",
        "track_id",
        "position",
      ]);
      expect(tableColumns(database, "play_history")).toEqual([
        "id",
        "library_id",
        "track_id",
        "played_at_unix_seconds",
      ]);
    } finally {
      database.close();
    }
  });

  it("reruns migrations without losing data", async () => {
    const dbPath = tempDatabasePath();
    await runSqliteMigrations(dbPath);

    const database = await openDatabase(dbPath);
    database.run(
      "INSERT INTO libraries (id, name, kind, location) VALUES (?, ?, ?, ?)",
      ["lib-1", "Music", "Filesystem", "/music"],
    );
    writeDatabase(dbPath, database);

    await runSqliteMigrations(dbPath);

    const migratedDatabase = await openDatabase(dbPath);
    try {
      expect(userVersion(migratedDatabase)).toBe(5);
      expect(countRows(migratedDatabase, "libraries")).toBe(1);
    } finally {
      migratedDatabase.close();
    }
  });

  it("adopts legacy inline schema databases without losing data", async () => {
    const dbPath = tempDatabasePath();
    const database = await createLegacyInlineDatabase();
    database.run(
      "INSERT INTO libraries (id, name, kind, location) VALUES (?, ?, ?, ?)",
      ["lib-1", "Music", "Filesystem", "/music"],
    );
    database.run(
      `INSERT INTO tracks (
        id, library_id, title, artist, duration_seconds, file_path, file_size, file_mtime
      ) VALUES (?, ?, ?, ?, ?, ?, ?, ?)`,
      ["trk-1", "lib-1", "Song", "Artist", 120, "/music/song.mp3", 1024, 42],
    );
    writeDatabase(dbPath, database);

    await runSqliteMigrations(dbPath);

    const migratedDatabase = await openDatabase(dbPath);
    try {
      expect(userVersion(migratedDatabase)).toBe(5);
      expect(countRows(migratedDatabase, "libraries")).toBe(1);
      expect(countRows(migratedDatabase, "tracks")).toBe(1);
      expect(tableColumns(migratedDatabase, "tracks")).toContain(
        "override_year",
      );
      expect(tableColumns(migratedDatabase, "play_history")).toEqual([
        "id",
        "library_id",
        "track_id",
        "played_at_unix_seconds",
      ]);
    } finally {
      migratedDatabase.close();
    }
  });
});

async function openDatabase(dbPath: string): Promise<Database> {
  const SQL = await initSqlJs();
  return new SQL.Database(readFileSync(dbPath));
}

async function createLegacyInlineDatabase(): Promise<Database> {
  const SQL = await initSqlJs();
  const database = new SQL.Database();
  database.run(`
    CREATE TABLE IF NOT EXISTS libraries (
      id       TEXT PRIMARY KEY NOT NULL,
      name     TEXT NOT NULL,
      kind     TEXT NOT NULL,
      location TEXT NOT NULL
    );

    CREATE TABLE IF NOT EXISTS tracks (
      id               TEXT PRIMARY KEY NOT NULL,
      library_id       TEXT NOT NULL,
      title            TEXT NOT NULL,
      artist           TEXT NOT NULL,
      duration_seconds INTEGER NOT NULL,
      file_path        TEXT NOT NULL,
      file_size        INTEGER NOT NULL,
      file_mtime       INTEGER NOT NULL,
      UNIQUE(library_id, file_path)
    );
  `);
  return database;
}

function tableColumns(database: Database, tableName: string): string[] {
  const result = database.exec(`PRAGMA table_info(${tableName})`);
  if (result.length === 0) {
    return [];
  }
  return result[0].values.map((row) => String(row[1]));
}

function userVersion(database: Database): number {
  const result = database.exec("PRAGMA user_version");
  return Number(result[0].values[0][0]);
}

function countRows(database: Database, tableName: string): number {
  const result = database.exec(`SELECT COUNT(*) FROM ${tableName}`);
  return Number(result[0].values[0][0]);
}

function writeDatabase(dbPath: string, database: Database): void {
  writeFileSync(dbPath, database.export());
  database.close();
}

function tempDatabasePath(): string {
  const dbPath = join(tmpdir(), `muzo-${Date.now()}-${Math.random()}.sqlite`);
  mkdirSync(join(dbPath, ".."), { recursive: true });
  return dbPath;
}
