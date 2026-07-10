import { readFileSync, writeFileSync } from "node:fs";

import initSqlJs, { type SqlJsStatic } from "sql.js";

import type {
  PlaylistDto,
  PlaylistEntryDto,
  PlaylistRepository,
} from "../../application/interfaces/playlist-interfaces";
import { locateSqlWasm } from "./sql-wasm-path";

let sqlModulePromise: Promise<SqlJsStatic> | null = null;

export class SqlitePlaylistRepository implements PlaylistRepository {
  constructor(private readonly dbPath: string) {}

  async add(playlist: PlaylistDto): Promise<void> {
    const SQL = await loadSqlModule();
    const database = new SQL.Database(readFileSync(this.dbPath));
    try {
      insertPlaylist(database, playlist);
      insertEntries(database, playlist);
      saveDatabase(database, this.dbPath);
    } finally {
      database.close();
    }
  }

  async findById(playlistId: string): Promise<PlaylistDto | null> {
    const SQL = await loadSqlModule();
    const database = new SQL.Database(readFileSync(this.dbPath));
    try {
      return findPlaylist(database, playlistId);
    } finally {
      database.close();
    }
  }

  async list(): Promise<PlaylistDto[]> {
    const SQL = await loadSqlModule();
    const database = new SQL.Database(readFileSync(this.dbPath));
    try {
      const statement = database.prepare(
        "SELECT id, name FROM playlists ORDER BY rowid",
      );
      try {
        const playlists: PlaylistDto[] = [];
        while (statement.step()) {
          const row = statement.getAsObject();
          playlists.push({
            id: stringColumn(row, "id"),
            name: stringColumn(row, "name"),
            entries: listEntries(database, stringColumn(row, "id")),
          });
        }
        return playlists;
      } finally {
        statement.free();
      }
    } finally {
      database.close();
    }
  }

  async save(playlist: PlaylistDto): Promise<void> {
    const SQL = await loadSqlModule();
    const database = new SQL.Database(readFileSync(this.dbPath));
    try {
      database.run("UPDATE playlists SET name = ? WHERE id = ?", [
        playlist.name,
        playlist.id,
      ]);
      database.run("DELETE FROM playlist_entries WHERE playlist_id = ?", [
        playlist.id,
      ]);
      insertEntries(database, playlist);
      saveDatabase(database, this.dbPath);
    } finally {
      database.close();
    }
  }
}

function findPlaylist(
  database: InstanceType<SqlJsStatic["Database"]>,
  playlistId: string,
): PlaylistDto | null {
  const statement = database.prepare("SELECT id, name FROM playlists WHERE id = ?");
  try {
    statement.bind([playlistId]);
    if (!statement.step()) {
      return null;
    }
    const row = statement.getAsObject();
    return {
      id: stringColumn(row, "id"),
      name: stringColumn(row, "name"),
      entries: listEntries(database, playlistId),
    };
  } finally {
    statement.free();
  }
}

function insertPlaylist(
  database: InstanceType<SqlJsStatic["Database"]>,
  playlist: PlaylistDto,
): void {
  database.run("INSERT INTO playlists (id, name) VALUES (?, ?)", [
    playlist.id,
    playlist.name,
  ]);
}

function insertEntries(
  database: InstanceType<SqlJsStatic["Database"]>,
  playlist: PlaylistDto,
): void {
  for (const entry of playlist.entries) {
    database.run(
      "INSERT INTO playlist_entries (id, playlist_id, track_id, position) VALUES (?, ?, ?, ?)",
      [entry.id, playlist.id, entry.trackId, entry.position],
    );
  }
}

function listEntries(
  database: InstanceType<SqlJsStatic["Database"]>,
  playlistId: string,
): PlaylistEntryDto[] {
  const statement = database.prepare(`
    SELECT
      playlist_entries.id,
      playlist_entries.track_id,
      playlist_entries.position,
      tracks.library_id AS track_library_id,
      tracks.title AS track_title,
      tracks.artist AS track_artist,
      tracks.duration_seconds AS track_duration_seconds
    FROM playlist_entries
    LEFT JOIN tracks ON tracks.id = playlist_entries.track_id
    WHERE playlist_entries.playlist_id = ?
    ORDER BY playlist_entries.position
  `);
  try {
    statement.bind([playlistId]);
    const entries: PlaylistEntryDto[] = [];
    while (statement.step()) {
      const row = statement.getAsObject();
      entries.push({
        id: stringColumn(row, "id"),
        trackId: stringColumn(row, "track_id"),
        trackTitle: nullableStringColumn(row, "track_title"),
        trackArtist: nullableStringColumn(row, "track_artist"),
        trackLibraryId: nullableStringColumn(row, "track_library_id"),
        trackDurationSeconds: nullableNumberColumn(row, "track_duration_seconds"),
        position: numberColumn(row, "position"),
      });
    }
    return entries;
  } finally {
    statement.free();
  }
}

function nullableStringColumn(
  row: Record<string, unknown>,
  column: string,
): string | null {
  const value = row[column];
  if (value === null || value === undefined) {
    return null;
  }
  if (typeof value !== "string") {
    throw new Error(`expected ${column} to be a string or null`);
  }
  return value;
}

function nullableNumberColumn(
  row: Record<string, unknown>,
  column: string,
): number | null {
  const value = row[column];
  if (value === null || value === undefined) {
    return null;
  }
  if (typeof value !== "number") {
    throw new Error(`expected ${column} to be a number or null`);
  }
  return value;
}

function loadSqlModule(): Promise<SqlJsStatic> {
  sqlModulePromise ??= initSqlJs({
    locateFile: locateSqlWasm,
  });
  return sqlModulePromise;
}

function saveDatabase(
  database: InstanceType<SqlJsStatic["Database"]>,
  dbPath: string,
): void {
  writeFileSync(dbPath, database.export());
}

function stringColumn(row: Record<string, unknown>, column: string): string {
  const value = row[column];
  if (typeof value !== "string") {
    throw new Error(`expected ${column} to be a string`);
  }
  return value;
}

function numberColumn(row: Record<string, unknown>, column: string): number {
  const value = row[column];
  if (typeof value !== "number") {
    throw new Error(`expected ${column} to be a number`);
  }
  return value;
}
