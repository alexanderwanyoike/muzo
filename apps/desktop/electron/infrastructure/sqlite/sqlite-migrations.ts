import { existsSync, mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { createRequire } from "node:module";
import { dirname } from "node:path";

import initSqlJs, { type SqlJsStatic } from "sql.js";

let sqlModulePromise: Promise<SqlJsStatic> | null = null;

const migrations = [
  {
    version: 1,
    name: "create-libraries",
    sql: `
      CREATE TABLE IF NOT EXISTS libraries (
        id       TEXT PRIMARY KEY NOT NULL,
        name     TEXT NOT NULL,
        kind     TEXT NOT NULL,
        location TEXT NOT NULL
      );
    `,
  },
  {
    version: 2,
    name: "create-tracks",
    sql: `
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
    `,
  },
  {
    version: 3,
    name: "track-metadata-overrides",
    sql: `
      ALTER TABLE tracks ADD COLUMN album TEXT;
      ALTER TABLE tracks ADD COLUMN track_number INTEGER;
      ALTER TABLE tracks ADD COLUMN disc_number INTEGER;
      ALTER TABLE tracks ADD COLUMN genre TEXT;
      ALTER TABLE tracks ADD COLUMN year INTEGER;
      ALTER TABLE tracks ADD COLUMN override_title TEXT;
      ALTER TABLE tracks ADD COLUMN override_artist TEXT;
      ALTER TABLE tracks ADD COLUMN override_album TEXT;
      ALTER TABLE tracks ADD COLUMN override_track_number INTEGER;
      ALTER TABLE tracks ADD COLUMN override_disc_number INTEGER;
      ALTER TABLE tracks ADD COLUMN override_genre TEXT;
      ALTER TABLE tracks ADD COLUMN override_year INTEGER;
    `,
  },
  {
    version: 4,
    name: "create-playlists",
    sql: `
      CREATE TABLE IF NOT EXISTS playlists (
        id TEXT PRIMARY KEY NOT NULL,
        name TEXT NOT NULL
      );

      CREATE TABLE IF NOT EXISTS playlist_entries (
        id TEXT PRIMARY KEY NOT NULL,
        playlist_id TEXT NOT NULL,
        track_id TEXT NOT NULL,
        position INTEGER NOT NULL,
        UNIQUE(playlist_id, position),
        FOREIGN KEY(playlist_id) REFERENCES playlists(id) ON DELETE CASCADE
      );
    `,
  },
  {
    version: 5,
    name: "create-play-history",
    sql: `
      CREATE TABLE IF NOT EXISTS play_history (
        id TEXT PRIMARY KEY NOT NULL,
        library_id TEXT NOT NULL,
        track_id TEXT NOT NULL,
        played_at_unix_seconds INTEGER NOT NULL
      );

      CREATE INDEX IF NOT EXISTS idx_play_history_library_track
      ON play_history(library_id, track_id);
    `,
  },
];

export async function runSqliteMigrations(dbPath: string): Promise<void> {
  const SQL = await loadSqlModule();
  const database = existsSync(dbPath)
    ? new SQL.Database(readFileSync(dbPath))
    : new SQL.Database();

  try {
    database.run(`
      CREATE TABLE IF NOT EXISTS electron_migrations (
        version INTEGER PRIMARY KEY NOT NULL,
        name TEXT NOT NULL,
        applied_at_unix_seconds INTEGER NOT NULL
      );
    `);

    let currentVersion = detectSchemaVersion(database);
    for (const migration of migrations) {
      if (migration.version <= currentVersion) {
        continue;
      }

      database.run("BEGIN");
      try {
        database.run(migration.sql);
        database.run(
          "INSERT OR IGNORE INTO electron_migrations (version, name, applied_at_unix_seconds) VALUES (?, ?, ?)",
          [migration.version, migration.name, Math.floor(Date.now() / 1000)],
        );
        setUserVersion(database, migration.version);
        database.run("COMMIT");
        currentVersion = migration.version;
      } catch (error) {
        database.run("ROLLBACK");
        throw error;
      }
    }

    mkdirSync(dirname(dbPath), { recursive: true });
    writeFileSync(dbPath, database.export());
  } finally {
    database.close();
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

function detectSchemaVersion(
  database: InstanceType<SqlJsStatic["Database"]>,
): number {
  const version = userVersion(database);
  if (version > 0) {
    return version;
  }

  if (tableExists(database, "play_history")) {
    return 5;
  }
  if (tableExists(database, "playlist_entries")) {
    return 4;
  }
  if (tableHasColumn(database, "tracks", "override_year")) {
    return 3;
  }
  if (tableExists(database, "tracks")) {
    return 2;
  }
  if (tableExists(database, "libraries")) {
    return 1;
  }

  return 0;
}

function userVersion(database: InstanceType<SqlJsStatic["Database"]>): number {
  const result = database.exec("PRAGMA user_version");
  return Number(result[0].values[0][0]);
}

function setUserVersion(
  database: InstanceType<SqlJsStatic["Database"]>,
  version: number,
): void {
  database.run(`PRAGMA user_version = ${version}`);
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

function tableHasColumn(
  database: InstanceType<SqlJsStatic["Database"]>,
  tableName: string,
  columnName: string,
): boolean {
  const result = database.exec(`PRAGMA table_info(${tableName})`);
  if (result.length === 0) {
    return false;
  }
  return result[0].values.some((row) => row[1] === columnName);
}
