import initSqlJs from "sql.js";
// @vitest-environment node
import { mkdtempSync, readFileSync, writeFileSync, rmSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { expect, it } from "vitest";
import { locateSqlWasm } from "./sql-wasm-path";
import { runSqliteMigrations } from "./sqlite-migrations";
import { SqliteLibraryRepository } from "./sqlite-library-repository";
import { SqliteDropboxLibrarySettings } from "./sqlite-dropbox-library-settings";

it("persists Dropbox library account bindings and the polling interval across restarts", async () => {
  const root = mkdtempSync(join(tmpdir(), "muzo-dropbox-settings-"));
  try {
    const dbPath = join(root, "muzo.sqlite");
    await runSqliteMigrations(dbPath);
    const settings = new SqliteDropboxLibrarySettings(dbPath);
    expect(await settings.intervalMinutes()).toBe(5);
    expect(await settings.accountFor("lib-1")).toBeNull();
    await settings.register({ id: "lib-1", kind: "dropbox", location: "/Music", name: "Cloud" }, "account-1");
    await settings.setIntervalMinutes(15);
    const reopened = new SqliteDropboxLibrarySettings(dbPath);
    expect(await reopened.accountFor("lib-1")).toBe("account-1");
    expect(await reopened.intervalMinutes()).toBe(15);
  } finally { rmSync(root, { recursive: true, force: true }); }
});

it("does not persist a library when its account binding cannot be written", async () => {
  const root = mkdtempSync(join(tmpdir(), "muzo-dropbox-atomic-"));
  try {
    const dbPath = join(root, "muzo.sqlite");
    await runSqliteMigrations(dbPath);
    const SQL = await initSqlJs({ locateFile: locateSqlWasm });
    const db = new SQL.Database(readFileSync(dbPath));
    db.run("CREATE TRIGGER reject_binding BEFORE INSERT ON dropbox_library_accounts BEGIN SELECT RAISE(ABORT, 'binding rejected'); END");
    writeFileSync(dbPath, db.export());
    db.close();
    const before = readFileSync(dbPath);
    const settings = new SqliteDropboxLibrarySettings(dbPath);
    await expect(settings.register({ id: "failed", kind: "dropbox", location: "/Music", name: "Cloud" }, "account")).rejects.toThrow("binding rejected");
    expect(await new SqliteLibraryRepository(dbPath).list()).toEqual([]);
    expect(await settings.accountFor("failed")).toBeNull();
    expect(readFileSync(dbPath)).toEqual(before);
  } finally { rmSync(root, { recursive: true, force: true }); }
});
