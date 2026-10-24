import { readFileSync, writeFileSync } from "node:fs";
import initSqlJs, { type Database } from "sql.js";
import type { DropboxLibrarySettings } from "../../application/interfaces/dropbox-library-settings";
import { locateSqlWasm } from "./sql-wasm-path";

const sqlModule = initSqlJs({ locateFile: locateSqlWasm });

export class SqliteDropboxLibrarySettings implements DropboxLibrarySettings {
  constructor(private readonly dbPath: string) {}

  async bind(libraryId: string, accountId: string): Promise<void> {
    await this.withDatabase((db) => {
      db.run("INSERT INTO dropbox_library_accounts (library_id, account_id) VALUES (?, ?)", [libraryId, accountId]);
    }, true);
  }

  async accountFor(libraryId: string): Promise<string | null> {
    return this.withDatabase((db) => {
      const result = db.exec("SELECT account_id FROM dropbox_library_accounts WHERE library_id = ?", [libraryId]);
      const value = result[0]?.values[0]?.[0];
      return typeof value === "string" ? value : null;
    });
  }

  async intervalMinutes(): Promise<number> {
    return this.withDatabase((db) => {
      const result = db.exec("SELECT interval_minutes FROM dropbox_sync_settings WHERE singleton = 1");
      return Number(result[0]?.values[0]?.[0] ?? 5);
    });
  }

  async setIntervalMinutes(minutes: number): Promise<void> {
    await this.withDatabase((db) => {
      db.run("UPDATE dropbox_sync_settings SET interval_minutes = ? WHERE singleton = 1", [minutes]);
    }, true);
  }

  private async withDatabase<T>(run: (db: Database) => T, save = false): Promise<T> {
    const SQL = await sqlModule;
    const db = new SQL.Database(readFileSync(this.dbPath));
    try {
      const result = run(db);
      if (save) writeFileSync(this.dbPath, db.export());
      return result;
    } finally { db.close(); }
  }
}
