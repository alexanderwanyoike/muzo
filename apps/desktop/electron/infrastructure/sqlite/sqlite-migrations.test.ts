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
      expect(userVersion(database)).toBe(8);
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
      expect(tableColumns(database, "dropbox_accounts")).toEqual([
        "account_id",
        "encrypted_access_token",
        "encrypted_refresh_token",
        "access_token_expires_at_unix_seconds",
        "connected_at_unix_seconds",
        "client_id",
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
      expect(userVersion(migratedDatabase)).toBe(8);
      expect(countRows(migratedDatabase, "libraries")).toBe(1);
    } finally {
      migratedDatabase.close();
    }
  });

  it.each([0, 4, 5])(
    "adopts a legacy Tauri database at version %s without losing data",
    async (version) => {
      const dbPath = tempDatabasePath();
      const SQL = await initSqlJs();
      const old = new SQL.Database();
      old.run(
        readFileSync(
          new URL("./fixtures/legacy-tauri-v5.sql", import.meta.url),
          "utf8",
        ),
      );
      if (version === 4) old.run("DROP TABLE play_history");
      old.run(`PRAGMA user_version = ${version}`);
      old.run(
        "INSERT INTO libraries (id, name, kind, location) VALUES ('legacy', 'Music', 'Filesystem', '/music')",
      );
      old.run(
        "INSERT INTO tracks (id, library_id, title, artist, duration_seconds, file_path, file_size, file_mtime, album, override_title) VALUES ('song', 'legacy', 'Song', 'Artist', 120, '/music/song.mp3', 100, 10, 'Album', 'Edited')",
      );
      writeDatabase(dbPath, old);
      await runSqliteMigrations(dbPath);
      const migrated = await openDatabase(dbPath);
      try {
        expect(userVersion(migrated)).toBeGreaterThan(5);
        expect(
          migrated.exec("SELECT album, override_title FROM tracks")[0].values,
        ).toEqual([["Album", "Edited"]]);
        expect(countRows(migrated, "libraries")).toBe(1);
        expect(countRows(migrated, "electron_migrations")).toBe(
          userVersion(migrated),
        );
      } finally {
        migrated.close();
      }
      await expect(runSqliteMigrations(dbPath)).resolves.toBeUndefined();
    },
  );

  it("leaves unsupported or inconsistent legacy databases untouched", async () => {
    for (const invalid of [
      "PRAGMA user_version = 99",
      "ALTER TABLE tracks DROP COLUMN album",
    ]) {
      const path = tempDatabasePath();
      const SQL = await initSqlJs();
      const database = new SQL.Database();
      database.run(
        readFileSync(
          new URL("./fixtures/legacy-tauri-v5.sql", import.meta.url),
          "utf8",
        ),
      );
      database.run(invalid);
      writeDatabase(path, database);
      const before = readFileSync(path);
      await expect(runSqliteMigrations(path)).rejects.toThrow(
        /legacy database/i,
      );
      expect(readFileSync(path)).toEqual(before);
    }
  });

  it("adopts the original unversioned library and track schema", async () => {
    const path = tempDatabasePath();
    const SQL = await initSqlJs();
    const database = new SQL.Database();
    const legacy = readFileSync(
      new URL("./fixtures/legacy-tauri-v5.sql", import.meta.url),
      "utf8",
    );
    database.run(legacy.slice(0, legacy.indexOf("ALTER TABLE")));
    database.run(
      "INSERT INTO libraries (id, name, kind, location) VALUES ('old', 'Original', 'Filesystem', '/music')",
    );
    writeDatabase(path, database);
    await runSqliteMigrations(path);
    const migrated = await openDatabase(path);
    try {
      expect(countRows(migrated, "libraries")).toBe(1);
      expect(tableColumns(migrated, "tracks")).toContain("album");
    } finally {
      migrated.close();
    }
  });
});

async function openDatabase(dbPath: string): Promise<Database> {
  const SQL = await initSqlJs();
  return new SQL.Database(readFileSync(dbPath));
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
