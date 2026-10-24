// @vitest-environment node

import { mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { join } from "node:path";
import { tmpdir } from "node:os";

import initSqlJs from "sql.js";
import { describe, expect, it } from "vitest";

import { SqliteTrackRepository } from "./sqlite-track-repository";
import { runSqliteMigrations } from "./sqlite-migrations";

describe("Electron track repository", () => {
  it("lists tracks for a library with displayed metadata and override state", async () => {
    const SQL = await initSqlJs();
    const database = new SQL.Database();
    database.run(`
      CREATE TABLE tracks (
        id TEXT PRIMARY KEY NOT NULL,
        library_id TEXT NOT NULL,
        title TEXT NOT NULL,
        artist TEXT NOT NULL,
        duration_seconds INTEGER NOT NULL,
        file_path TEXT NOT NULL,
        file_size INTEGER NOT NULL,
        file_mtime INTEGER NOT NULL,
        album TEXT,
        track_number INTEGER,
        disc_number INTEGER,
        genre TEXT,
        year INTEGER,
        override_title TEXT,
        override_artist TEXT,
        override_album TEXT,
        override_track_number INTEGER,
        override_disc_number INTEGER,
        override_genre TEXT,
        override_year INTEGER
      );
    `);
    database.run(
      `INSERT INTO tracks (
        id, library_id, title, artist, album, track_number, disc_number, genre, year,
        override_title, override_album, duration_seconds, file_path, file_size, file_mtime
      ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)`,
      [
        "trk-1",
        "lib-1",
        "File Title",
        "File Artist",
        "File Album",
        7,
        2,
        "Rock",
        1999,
        "Edited Title",
        "Edited Album",
        200,
        "/music/a.mp3",
        1000,
        123,
      ],
    );
    database.run(
      `INSERT INTO tracks (
        id, library_id, title, artist, duration_seconds, file_path, file_size, file_mtime
      ) VALUES (?, ?, ?, ?, ?, ?, ?, ?)`,
      ["trk-2", "lib-2", "Other", "Artist", 120, "/music/b.mp3", 500, 456],
    );

    const dbPath = tempDatabasePath();
    mkdirSync(join(dbPath, ".."), { recursive: true });
    writeFileSync(dbPath, database.export());

    await expect(new SqliteTrackRepository(dbPath).listForLibrary("lib-1")).resolves.toEqual([
      {
        id: "trk-1",
        libraryId: "lib-1",
        title: "Edited Title",
        artist: "File Artist",
        album: "Edited Album",
        trackNumber: 7,
        discNumber: 2,
        genre: "Rock",
        year: 1999,
        metadataOverridden: true,
        durationSeconds: 200,
        filePath: "/music/a.mp3",
      },
    ]);
  });

  it("returns an empty list before the database has been created", async () => {
    await expect(
      new SqliteTrackRepository("/path/that/does/not/exist/muzo.sqlite").listForLibrary(
        "lib-1",
      ),
    ).resolves.toEqual([]);
  });

  it("returns an empty list before tracks have been scanned", async () => {
    const SQL = await initSqlJs();
    const database = new SQL.Database();
    database.run(`
      CREATE TABLE libraries (
        id TEXT PRIMARY KEY NOT NULL,
        name TEXT NOT NULL,
        kind TEXT NOT NULL,
        location TEXT NOT NULL
      );
    `);

    const dbPath = tempDatabasePath();
    mkdirSync(join(dbPath, ".."), { recursive: true });
    writeFileSync(dbPath, database.export());

    await expect(
      new SqliteTrackRepository(dbPath).listForLibrary("lib-1"),
    ).resolves.toEqual([]);
  });

  it("upserts scanned tracks without overwriting metadata overrides", async () => {
    const dbPath = tempDatabasePath();
    await runSqliteMigrations(dbPath);
    const repository = new SqliteTrackRepository(dbPath);

    await repository.upsertScannedTrack({
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
      filePath: "/music/song.mp3",
      fileSize: 1000,
      fileMtime: 123,
    });
    await setOverrideTitle(dbPath, "trk-1", "Edited Title");

    await repository.upsertScannedTrack({
      id: "trk-new",
      libraryId: "lib-1",
      title: "Updated File Title",
      artist: "Updated Artist",
      album: "Updated Album",
      trackNumber: 2,
      discNumber: 1,
      genre: "Rock",
      year: 2001,
      durationSeconds: 180,
      filePath: "/music/song.mp3",
      fileSize: 2000,
      fileMtime: 456,
    });

    await expect(repository.listForLibrary("lib-1")).resolves.toEqual([
      {
        id: "trk-1",
        libraryId: "lib-1",
        title: "Edited Title",
        artist: "Updated Artist",
        album: "Updated Album",
        trackNumber: 2,
        discNumber: 1,
        genre: "Rock",
        year: 2001,
        metadataOverridden: true,
        durationSeconds: 180,
        filePath: "/music/song.mp3",
      },
    ]);
  });

  it("deletes tracks by library and path", async () => {
    const dbPath = tempDatabasePath();
    await runSqliteMigrations(dbPath);
    const repository = new SqliteTrackRepository(dbPath);
    await repository.upsertScannedTrack({
      id: "trk-1",
      libraryId: "lib-1",
      title: "Song",
      artist: "Artist",
      album: null,
      trackNumber: null,
      discNumber: null,
      genre: null,
      year: null,
      durationSeconds: 120,
      filePath: "/music/song.mp3",
      fileSize: 1000,
      fileMtime: 123,
    });

    await repository.deleteByLibraryAndPath("lib-1", "/music/song.mp3");

    await expect(repository.listForLibrary("lib-1")).resolves.toEqual([]);
  });

  it("updates metadata overrides displayed for a track", async () => {
    const dbPath = tempDatabasePath();
    await runSqliteMigrations(dbPath);
    const repository = new SqliteTrackRepository(dbPath);
    await repository.upsertScannedTrack({
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
      filePath: "/music/song.mp3",
      fileSize: 1000,
      fileMtime: 123,
    });

    await repository.updateMetadataOverride("lib-1", "trk-1", {
      title: "Edited Title",
      artist: null,
      album: "Edited Album",
      trackNumber: 4,
      discNumber: 1,
      genre: "Soul",
      year: 1971,
    });

    await expect(repository.listForLibrary("lib-1")).resolves.toEqual([
      {
        id: "trk-1",
        libraryId: "lib-1",
        title: "Edited Title",
        artist: "File Artist",
        album: "Edited Album",
        trackNumber: 4,
        discNumber: 1,
        genre: "Soul",
        year: 1971,
        metadataOverridden: true,
        durationSeconds: 120,
        filePath: "/music/song.mp3",
      },
    ]);
  });

  it("clears metadata overrides so file metadata is displayed again", async () => {
    const dbPath = tempDatabasePath();
    await runSqliteMigrations(dbPath);
    const repository = new SqliteTrackRepository(dbPath);
    await repository.upsertScannedTrack({
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
      filePath: "/music/song.mp3",
      fileSize: 1000,
      fileMtime: 123,
    });
    await repository.updateMetadataOverride("lib-1", "trk-1", {
      title: "Edited Title",
      artist: null,
      album: null,
      trackNumber: null,
      discNumber: null,
      genre: null,
      year: null,
    });

    await repository.clearMetadataOverride("lib-1", "trk-1");

    await expect(repository.listForLibrary("lib-1")).resolves.toEqual([
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
        filePath: "/music/song.mp3",
      },
    ]);
  });
});

async function setOverrideTitle(
  dbPath: string,
  trackId: string,
  title: string,
): Promise<void> {
  const SQL = await initSqlJs();
  const database = new SQL.Database(readFileSync(dbPath));
  database.run("UPDATE tracks SET override_title = ? WHERE id = ?", [
    title,
    trackId,
  ]);
  writeFileSync(dbPath, database.export());
  database.close();
}

function tempDatabasePath(): string {
  return join(tmpdir(), `muzo-${Date.now()}-${Math.random()}.sqlite`);
}
