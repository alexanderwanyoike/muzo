import { existsSync, readFileSync, writeFileSync } from "node:fs";

import initSqlJs, { type SqlJsStatic } from "sql.js";

import type {
  DropboxAccountRepository,
  DropboxTokenCipher,
  StoredDropboxAccount,
} from "../../application/interfaces/dropbox-account-interfaces";
import { locateSqlWasm } from "./sql-wasm-path";

let sqlModulePromise: Promise<SqlJsStatic> | null = null;

export class SqliteDropboxAccountRepository
  implements DropboxAccountRepository
{
  constructor(
    private readonly dbPath: string,
    private readonly cipher: DropboxTokenCipher,
  ) {}

  async disconnect(): Promise<void> {
    const SQL = await loadSqlModule();
    const database = this.openDatabase(SQL);
    try {
      if (tableExists(database, "dropbox_accounts")) {
        database.run("DELETE FROM dropbox_accounts");
        saveDatabase(database, this.dbPath);
      }
    } finally {
      database.close();
    }
  }

  async find(): Promise<StoredDropboxAccount | null> {
    if (!existsSync(this.dbPath)) {
      return null;
    }

    const SQL = await loadSqlModule();
    const database = new SQL.Database(readFileSync(this.dbPath));
    try {
      if (!tableExists(database, "dropbox_accounts")) {
        return null;
      }

      const statement = database.prepare(`
        SELECT
          account_id,
          encrypted_access_token,
          encrypted_refresh_token,
          access_token_expires_at_unix_seconds,
          connected_at_unix_seconds
        FROM dropbox_accounts
        ORDER BY connected_at_unix_seconds DESC
        LIMIT 1
      `);
      try {
        if (!statement.step()) {
          return null;
        }
        return accountFromRow(statement.getAsObject(), this.cipher);
      } finally {
        statement.free();
      }
    } finally {
      database.close();
    }
  }

  async save(account: StoredDropboxAccount): Promise<void> {
    const SQL = await loadSqlModule();
    const database = this.openDatabase(SQL);
    try {
      database.run("DELETE FROM dropbox_accounts");
      database.run(
        `
          INSERT INTO dropbox_accounts (
            account_id,
            encrypted_access_token,
            encrypted_refresh_token,
            access_token_expires_at_unix_seconds,
            connected_at_unix_seconds
          ) VALUES (?, ?, ?, ?, ?)
        `,
        [
          account.accountId,
          this.cipher.protect(account.accessToken),
          this.cipher.protect(account.refreshToken),
          account.accessTokenExpiresAtUnixSeconds,
          account.connectedAtUnixSeconds,
        ],
      );
      saveDatabase(database, this.dbPath);
    } finally {
      database.close();
    }
  }

  private openDatabase(SQL: SqlJsStatic): InstanceType<SqlJsStatic["Database"]> {
    return existsSync(this.dbPath)
      ? new SQL.Database(readFileSync(this.dbPath))
      : new SQL.Database();
  }
}

function accountFromRow(
  row: Record<string, unknown>,
  cipher: DropboxTokenCipher,
): StoredDropboxAccount {
  return {
    accountId: stringColumn(row, "account_id"),
    accessToken: cipher.unprotect(stringColumn(row, "encrypted_access_token")),
    refreshToken: cipher.unprotect(stringColumn(row, "encrypted_refresh_token")),
    accessTokenExpiresAtUnixSeconds: numberColumn(
      row,
      "access_token_expires_at_unix_seconds",
    ),
    connectedAtUnixSeconds: numberColumn(row, "connected_at_unix_seconds"),
  };
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
