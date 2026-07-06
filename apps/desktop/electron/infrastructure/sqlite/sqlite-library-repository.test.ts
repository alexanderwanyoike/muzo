// @vitest-environment node

import { existsSync, mkdirSync, writeFileSync } from "node:fs";
import { join } from "node:path";
import { tmpdir } from "node:os";

import initSqlJs from "sql.js";
import { describe, expect, it } from "vitest";

import { SqliteLibraryRepository } from "./sqlite-library-repository";
import { runSqliteMigrations } from "./sqlite-migrations";

describe("Electron library repository", () => {
  it("lists libraries from the existing Muzo SQLite schema", async () => {
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
    database.run(
      "INSERT INTO libraries (id, name, kind, location) VALUES (?, ?, ?, ?)",
      ["lib-1", "Local Music", "Filesystem", "/music"],
    );
    database.run(
      "INSERT INTO libraries (id, name, kind, location) VALUES (?, ?, ?, ?)",
      ["lib-2", "Dropbox", "Dropbox", "/Music"],
    );

    const dbPath = join(tmpdir(), `muzo-${Date.now()}-${Math.random()}.sqlite`);
    mkdirSync(join(dbPath, ".."), { recursive: true });
    writeFileSync(dbPath, database.export());

    const repository = new SqliteLibraryRepository(dbPath);

    await expect(repository.list()).resolves.toEqual([
      {
        id: "lib-1",
        name: "Local Music",
        kind: "filesystem",
        location: "/music",
      },
      {
        id: "lib-2",
        name: "Dropbox",
        kind: "dropbox",
        location: "/Music",
      },
    ]);
  });

  it("returns an empty list before the database has been created", async () => {
    await expect(
      new SqliteLibraryRepository("/path/that/does/not/exist/muzo.sqlite").list(),
    ).resolves.toEqual([]);
  });

  it("finds a library by id", async () => {
    const dbPath = tempDatabasePath();
    await runSqliteMigrations(dbPath);
    const repository = new SqliteLibraryRepository(dbPath);
    await repository.add({
      id: "lib-1",
      name: "Local Music",
      kind: "filesystem",
      location: "/music",
    });

    await expect(repository.findById("lib-1")).resolves.toEqual({
      id: "lib-1",
      name: "Local Music",
      kind: "filesystem",
      location: "/music",
    });
    await expect(repository.findById("missing")).resolves.toBeNull();
  });

  it("persists a new library after migrations have created the schema", async () => {
    const dbPath = tempDatabasePath();
    await runSqliteMigrations(dbPath);
    const repository = new SqliteLibraryRepository(dbPath);

    await repository.add({
      id: "lib-1",
      name: "Local Music",
      kind: "filesystem",
      location: "/music",
    });

    expect(existsSync(dbPath)).toBe(true);
    await expect(new SqliteLibraryRepository(dbPath).list()).resolves.toEqual([
      {
        id: "lib-1",
        name: "Local Music",
        kind: "filesystem",
        location: "/music",
      },
    ]);
  });

  it("does not create schema as a repository side effect", async () => {
    const dbPath = tempDatabasePath();
    const repository = new SqliteLibraryRepository(dbPath);

    await expect(
      repository.add({
        id: "lib-1",
        name: "Local Music",
        kind: "filesystem",
        location: "/music",
      }),
    ).rejects.toThrow("no such table: libraries");
  });
});

function tempDatabasePath(): string {
  return join(tmpdir(), `muzo-${Date.now()}-${Math.random()}.sqlite`);
}
