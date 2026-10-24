import { readFileSync, readdirSync } from "node:fs";
import { join } from "node:path";
import type { Database, SqlJsStatic } from "sql.js";

const lastTauriMigration = 5;

export function legacyMigrationNames(database: Database, SQL: SqlJsStatic, directory: string): string[] {
  if (hasTable(database, "electron_migrations") || !hasTable(database, "libraries")) return [];
  const version = Number(database.exec("PRAGMA user_version")[0]?.values[0]?.[0] ?? 0);
  const names = readdirSync(directory).filter((name) => /^000[1-5]-.*\.sql$/.test(name)).sort();
  if (version < 0 || version > lastTauriMigration) throw new Error("Unsupported legacy database version");
  const candidates = version ? [version] : [5, 4, 3, 2, 1];
  for (const candidate of candidates) {
    const expected = new SQL.Database();
    try {
      for (const name of names.slice(0, candidate)) expected.run(readFileSync(join(directory, name), "utf8"));
      const tables = ["libraries", "tracks", "playlists", "playlist_entries", "play_history"];
      if (tables.every((table) => schemaSignature(database, table) === schemaSignature(expected, table))) {
        return names.slice(0, candidate);
      }
    } finally { expected.close(); }
  }
  throw new Error("Legacy database schema does not match its migration version");
}

function hasTable(database: Database, table: string): boolean {
  return database.exec("SELECT name FROM sqlite_master WHERE type = 'table' AND name = ?", [table]).length > 0;
}

function schemaSignature(database: Database, table: string): string {
  const columns = database.exec(`PRAGMA table_info(${table})`)[0]?.values ?? [];
  const indexes = database.exec(`PRAGMA index_list(${table})`)[0]?.values ?? [];
  const uniqueColumns = indexes.filter((row) => row[2] === 1).map((row) => {
    const name = String(row[1]).replaceAll('"', '""');
    return (database.exec(`PRAGMA index_info("${name}")`)[0]?.values ?? []).map((entry) => entry[2]);
  });
  return JSON.stringify({ columns, uniqueColumns });
}
