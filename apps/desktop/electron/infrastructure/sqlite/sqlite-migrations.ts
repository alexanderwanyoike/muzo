import {
  existsSync,
  mkdirSync,
  readFileSync,
  readdirSync,
  writeFileSync,
} from "node:fs";
import { createRequire } from "node:module";
import { basename, dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

import initSqlJs, { type SqlJsStatic } from "sql.js";
import { Umzug, type MigrationParams, type UmzugStorage } from "umzug";

let sqlModulePromise: Promise<SqlJsStatic> | null = null;

interface MigrationContext {
  database: InstanceType<SqlJsStatic["Database"]>;
}

const migrationsDir = join(dirname(fileURLToPath(import.meta.url)), "migrations");

export async function runSqliteMigrations(dbPath: string): Promise<void> {
  const SQL = await loadSqlModule();
  const database = existsSync(dbPath)
    ? new SQL.Database(readFileSync(dbPath))
    : new SQL.Database();

  try {
    ensureMigrationTable(database);
    adoptLegacySchema(database);

    const migrator = new Umzug<MigrationContext>({
      context: { database },
      logger: undefined,
      migrations: migrationFiles().map((path) => ({
        name: migrationName(path),
        path,
        up: async ({ context }) => {
          context.database.run(readFileSync(path, "utf8"));
          setUserVersion(context.database, migrationVersion(path));
        },
      })),
      storage: new SqliteMigrationStorage(database),
    });

    await migrator.up();

    mkdirSync(dirname(dbPath), { recursive: true });
    writeFileSync(dbPath, database.export());
  } finally {
    database.close();
  }
}

class SqliteMigrationStorage implements UmzugStorage<MigrationContext> {
  constructor(private readonly database: InstanceType<SqlJsStatic["Database"]>) {}

  async executed(): Promise<string[]> {
    const result = this.database.exec(
      "SELECT version FROM electron_migrations ORDER BY version",
    );
    if (result.length === 0) {
      return [];
    }
    const migrationNames = migrationNamesByVersion();
    return result[0].values
      .map((row) => migrationNames.get(Number(row[0])))
      .filter((name): name is string => name !== undefined);
  }

  async logMigration({ name }: MigrationParams<MigrationContext>): Promise<void> {
    this.database.run(
      "INSERT OR REPLACE INTO electron_migrations (version, name, applied_at_unix_seconds) VALUES (?, ?, ?)",
      [migrationVersion(name), name, Math.floor(Date.now() / 1000)],
    );
  }

  async unlogMigration({
    name,
  }: MigrationParams<MigrationContext>): Promise<void> {
    this.database.run("DELETE FROM electron_migrations WHERE name = ?", [name]);
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

function migrationFiles(): string[] {
  return readdirSync(migrationsDir)
    .filter((file) => file.endsWith(".sql"))
    .sort()
    .map((file) => join(migrationsDir, file));
}

function migrationName(path: string): string {
  return basename(path);
}

function migrationNamesByVersion(): Map<number, string> {
  return new Map(
    migrationFiles().map((path) => [migrationVersion(path), migrationName(path)]),
  );
}

function migrationVersion(nameOrPath: string): number {
  const name = migrationName(nameOrPath);
  const match = /^(\d{4})-/.exec(name);
  if (!match) {
    throw new Error(`invalid migration filename: ${name}`);
  }
  return Number(match[1]);
}

function ensureMigrationTable(
  database: InstanceType<SqlJsStatic["Database"]>,
): void {
  database.run(`
    CREATE TABLE IF NOT EXISTS electron_migrations (
      version INTEGER PRIMARY KEY NOT NULL,
      name TEXT NOT NULL,
      applied_at_unix_seconds INTEGER NOT NULL
    );
  `);
}

function adoptLegacySchema(
  database: InstanceType<SqlJsStatic["Database"]>,
): void {
  const version = userVersion(database);
  const detectedVersion = Math.max(version, detectLegacySchemaVersion(database));
  if (detectedVersion === 0) {
    return;
  }

  for (const path of migrationFiles()) {
    const versionFromPath = migrationVersion(path);
    if (versionFromPath <= detectedVersion) {
      database.run(
        "INSERT OR REPLACE INTO electron_migrations (version, name, applied_at_unix_seconds) VALUES (?, ?, ?)",
        [versionFromPath, migrationName(path), Math.floor(Date.now() / 1000)],
      );
    }
  }
  setUserVersion(database, detectedVersion);
}

function detectLegacySchemaVersion(
  database: InstanceType<SqlJsStatic["Database"]>,
): number {
  if (tableExists(database, "play_history")) {
    return 5;
  }
  if (tableExists(database, "playlist_entries")) {
    return 4;
  }
  if (tableHasColumn(database, "tracks", "override_year")) {
    return 3;
  }
  if (tableExists(database, "tracks")) {
    return 2;
  }
  if (tableExists(database, "libraries")) {
    return 1;
  }

  return 0;
}

function userVersion(database: InstanceType<SqlJsStatic["Database"]>): number {
  const result = database.exec("PRAGMA user_version");
  return Number(result[0].values[0][0]);
}

function setUserVersion(
  database: InstanceType<SqlJsStatic["Database"]>,
  version: number,
): void {
  database.run(`PRAGMA user_version = ${version}`);
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

function tableHasColumn(
  database: InstanceType<SqlJsStatic["Database"]>,
  tableName: string,
  columnName: string,
): boolean {
  const result = database.exec(`PRAGMA table_info(${tableName})`);
  if (result.length === 0) {
    return false;
  }
  return result[0].values.some((row) => row[1] === columnName);
}
