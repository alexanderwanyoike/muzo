import {
  existsSync,
  mkdirSync,
  readFileSync,
  writeFileSync,
} from "node:fs";
import { createRequire } from "node:module";
import { basename, dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

import initSqlJs, { type SqlJsStatic } from "sql.js";
import {
  Umzug,
  type MigrationParams,
  type UmzugStorage,
} from "umzug";

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
    const migrator = createMigrator(database);
    await migrator.up();

    mkdirSync(dirname(dbPath), { recursive: true });
    writeFileSync(dbPath, database.export());
  } finally {
    database.close();
  }
}

function createMigrator(
  database: InstanceType<SqlJsStatic["Database"]>,
): Umzug<MigrationContext> {
  return new Umzug<MigrationContext>({
    context: { database },
    logger: undefined,
    migrations: {
      glob: ["*.sql", { cwd: migrationsDir }],
      resolve: ({ name, path }) => ({
        name,
        path,
        up: async ({ context }) => {
          if (!path) {
            throw new Error(`missing migration path for ${name}`);
          }
          context.database.run(readFileSync(path, "utf8"));
          setUserVersion(context.database, migrationVersion(name));
        },
      }),
    },
    storage: new SqliteMigrationStorage(database),
  });
}

class SqliteMigrationStorage implements UmzugStorage<MigrationContext> {
  constructor(private readonly database: InstanceType<SqlJsStatic["Database"]>) {}

  async executed(): Promise<string[]> {
    this.ensureTable();
    const result = this.database.exec(
      "SELECT name FROM electron_migrations ORDER BY version",
    );
    if (result.length === 0) {
      return [];
    }
    return result[0].values.map((row) => String(row[0]));
  }

  async logMigration({ name }: MigrationParams<MigrationContext>): Promise<void> {
    this.ensureTable();
    this.database.run(
      "INSERT OR REPLACE INTO electron_migrations (version, name, applied_at_unix_seconds) VALUES (?, ?, ?)",
      [migrationVersion(name), name, Math.floor(Date.now() / 1000)],
    );
  }

  async unlogMigration({
    name,
  }: MigrationParams<MigrationContext>): Promise<void> {
    this.ensureTable();
    this.database.run("DELETE FROM electron_migrations WHERE name = ?", [name]);
  }

  private ensureTable(): void {
    this.database.run(`
      CREATE TABLE IF NOT EXISTS electron_migrations (
        version INTEGER PRIMARY KEY NOT NULL,
        name TEXT NOT NULL,
        applied_at_unix_seconds INTEGER NOT NULL
      );
    `);
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

function migrationName(path: string): string {
  return basename(path);
}

function migrationVersion(nameOrPath: string): number {
  const name = migrationName(nameOrPath);
  const match = /^(\d{4})-/.exec(name);
  if (!match) {
    throw new Error(`invalid migration filename: ${name}`);
  }
  return Number(match[1]);
}

function setUserVersion(
  database: InstanceType<SqlJsStatic["Database"]>,
  version: number,
): void {
  database.run(`PRAGMA user_version = ${version}`);
}
