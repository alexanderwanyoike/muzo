import { readFileSync, writeFileSync } from "node:fs";
import initSqlJs from "sql.js";
import type { LibraryRemover } from "../../application/interfaces/library-removal";
import { locateSqlWasm } from "./sql-wasm-path";

const sqlModule = initSqlJs({ locateFile: locateSqlWasm });

export class SqliteLibraryRemover implements LibraryRemover {
  constructor(private readonly dbPath: string) {}

  async remove(libraryId: string): Promise<boolean> {
    const SQL = await sqlModule;
    const database = new SQL.Database(readFileSync(this.dbPath));
    try {
      database.run("BEGIN TRANSACTION");
      database.run("DELETE FROM play_history WHERE library_id = ?", [libraryId]);
      database.run("DELETE FROM tracks WHERE library_id = ?", [libraryId]);
      database.run("DELETE FROM dropbox_library_accounts WHERE library_id = ?", [libraryId]);
      database.run("DELETE FROM libraries WHERE id = ?", [libraryId]);
      const removed = database.getRowsModified() > 0;
      database.run("COMMIT");
      writeFileSync(this.dbPath, database.export());
      return removed;
    } finally {
      database.close();
    }
  }
}
