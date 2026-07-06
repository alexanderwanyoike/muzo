import { existsSync, mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { createRequire } from "node:module";
import { dirname } from "node:path";

import initSqlJs, { type SqlJsStatic } from "sql.js";

import type { LibraryDto, LibraryKindDto } from "../../../src/types";
import type { LibraryRepository } from "../../application/interfaces/repository-interfaces";

let sqlModulePromise: Promise<SqlJsStatic> | null = null;

export class SqliteLibraryRepository implements LibraryRepository {
  constructor(private readonly dbPath: string) {}

  async add(library: LibraryDto): Promise<void> {
    const SQL = await loadSqlModule();
    const database = this.openDatabase(SQL);
    try {
      database.run(
        "INSERT INTO libraries (id, name, kind, location) VALUES (?, ?, ?, ?)",
        [
          library.id,
          library.name,
          libraryKindToDatabase(library.kind),
          library.location,
        ],
      );
      this.saveDatabase(database);
    } finally {
      database.close();
    }
  }

  async list(): Promise<LibraryDto[]> {
    if (!existsSync(this.dbPath)) {
      return [];
    }

    const SQL = await loadSqlModule();
    const database = new SQL.Database(readFileSync(this.dbPath));
    try {
      const statement = database.prepare(
        "SELECT id, name, kind, location FROM libraries ORDER BY rowid",
      );
      try {
        const libraries: LibraryDto[] = [];
        while (statement.step()) {
          const row = statement.getAsObject() as Record<string, unknown>;
          libraries.push({
            id: stringColumn(row, "id"),
            name: stringColumn(row, "name"),
            kind: libraryKindFromDatabase(stringColumn(row, "kind")),
            location: stringColumn(row, "location"),
          });
        }
        return libraries;
      } finally {
        statement.free();
      }
    } finally {
      database.close();
    }
  }

  private openDatabase(SQL: SqlJsStatic): InstanceType<SqlJsStatic["Database"]> {
    return existsSync(this.dbPath)
      ? new SQL.Database(readFileSync(this.dbPath))
      : new SQL.Database();
  }

  private saveDatabase(database: InstanceType<SqlJsStatic["Database"]>): void {
    mkdirSync(dirname(this.dbPath), { recursive: true });
    writeFileSync(this.dbPath, database.export());
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

function stringColumn(row: Record<string, unknown>, column: string): string {
  const value = row[column];
  if (typeof value !== "string") {
    throw new Error(`expected ${column} to be a string`);
  }
  return value;
}

function libraryKindFromDatabase(kind: string): LibraryKindDto {
  switch (kind) {
    case "Filesystem":
      return "filesystem";
    case "Dropbox":
      return "dropbox";
    default:
      throw new Error(`unknown library kind in database: ${kind}`);
  }
}

function libraryKindToDatabase(kind: LibraryKindDto): string {
  switch (kind) {
    case "filesystem":
      return "Filesystem";
    case "dropbox":
      return "Dropbox";
  }
}
