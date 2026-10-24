import { existsSync, mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { dirname } from "node:path";

import initSqlJs, { type SqlJsStatic } from "sql.js";

import type { LibraryRecord, LibraryKind } from "../../application/interfaces/library-record";
import type { LibraryRepository } from "../../application/interfaces/repository-interfaces";
import { locateSqlWasm } from "./sql-wasm-path";

let sqlModulePromise: Promise<SqlJsStatic> | null = null;

export class SqliteLibraryRepository implements LibraryRepository {
  constructor(private readonly dbPath: string) {}

  async add(library: LibraryRecord): Promise<void> {
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

  async findById(libraryId: string): Promise<LibraryRecord | null> {
    if (!existsSync(this.dbPath)) {
      return null;
    }

    const SQL = await loadSqlModule();
    const database = new SQL.Database(readFileSync(this.dbPath));
    try {
      const statement = database.prepare(
        "SELECT id, name, kind, location FROM libraries WHERE id = ?",
      );
      try {
        statement.bind([libraryId]);
        if (!statement.step()) {
          return null;
        }
        return libraryFromRow(statement.getAsObject());
      } finally {
        statement.free();
      }
    } finally {
      database.close();
    }
  }

  async list(): Promise<LibraryRecord[]> {
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
        const libraries: LibraryRecord[] = [];
        while (statement.step()) {
          libraries.push(libraryFromRow(statement.getAsObject()));
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

function libraryFromRow(row: Record<string, unknown>): LibraryRecord {
  return {
    id: stringColumn(row, "id"),
    name: stringColumn(row, "name"),
    kind: libraryKindFromDatabase(stringColumn(row, "kind")),
    location: stringColumn(row, "location"),
  };
}

function loadSqlModule(): Promise<SqlJsStatic> {
  sqlModulePromise ??= initSqlJs({
    locateFile: locateSqlWasm,
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

function libraryKindFromDatabase(kind: string): LibraryKind {
  switch (kind) {
    case "Filesystem":
      return "filesystem";
    case "Dropbox":
      return "dropbox";
    default:
      throw new Error(`unknown library kind in database: ${kind}`);
  }
}

function libraryKindToDatabase(kind: LibraryKind): string {
  switch (kind) {
    case "filesystem":
      return "Filesystem";
    case "dropbox":
      return "Dropbox";
  }
}
