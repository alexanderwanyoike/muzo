// @vitest-environment node
import { mkdtempSync, rmSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { expect, it } from "vitest";
import { runSqliteMigrations } from "./sqlite-migrations";
import { SqliteLibraryRepository } from "./sqlite-library-repository";
import { SqliteDropboxLibrarySettings } from "./sqlite-dropbox-library-settings";

it("persists Dropbox library account bindings and the polling interval across restarts", async () => {
  const root = mkdtempSync(join(tmpdir(), "muzo-dropbox-settings-"));
  try {
    const dbPath = join(root, "muzo.sqlite");
    await runSqliteMigrations(dbPath);
    await new SqliteLibraryRepository(dbPath).add({ id: "lib-1", kind: "dropbox", location: "/Music", name: "Cloud" });
    const settings = new SqliteDropboxLibrarySettings(dbPath);
    expect(await settings.intervalMinutes()).toBe(5);
    expect(await settings.accountFor("lib-1")).toBeNull();
    await settings.bind("lib-1", "account-1");
    await settings.setIntervalMinutes(15);
    const reopened = new SqliteDropboxLibrarySettings(dbPath);
    expect(await reopened.accountFor("lib-1")).toBe("account-1");
    expect(await reopened.intervalMinutes()).toBe(15);
  } finally { rmSync(root, { recursive: true, force: true }); }
});
