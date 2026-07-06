// @vitest-environment node

import { mkdirSync, writeFileSync } from "node:fs";
import { join } from "node:path";
import { tmpdir } from "node:os";

import initSqlJs from "sql.js";
import { describe, expect, it } from "vitest";

import { SqliteLibraryRepository } from "./sqlite-library-repository";

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
});
