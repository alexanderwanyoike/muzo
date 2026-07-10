// @vitest-environment node

import { readFileSync } from "node:fs";
import { join } from "node:path";
import { tmpdir } from "node:os";

import initSqlJs from "sql.js";
import { describe, expect, it } from "vitest";

import type { DropboxTokenCipher } from "../../application/interfaces/dropbox-account-interfaces";
import { runSqliteMigrations } from "./sqlite-migrations";
import { SqliteDropboxAccountRepository } from "./sqlite-dropbox-account-repository";

describe("Electron Dropbox account repository", () => {
  it("stores encrypted Dropbox account tokens", async () => {
    const dbPath = tempDatabasePath();
    await runSqliteMigrations(dbPath);
    const cipher = new ReversingCipher();
    const repository = new SqliteDropboxAccountRepository(dbPath, cipher);

    await repository.save({
      accountId: "dbid:account-1",
      accessToken: "access-token-1",
      refreshToken: "refresh-token-1",
      accessTokenExpiresAtUnixSeconds: 1_780_000_000,
      connectedAtUnixSeconds: 1_770_000_000,
    });

    await expect(repository.find()).resolves.toEqual({
      accountId: "dbid:account-1",
      accessToken: "access-token-1",
      refreshToken: "refresh-token-1",
      accessTokenExpiresAtUnixSeconds: 1_780_000_000,
      connectedAtUnixSeconds: 1_770_000_000,
    });
    expect(await storedTokenColumns(dbPath)).toEqual({
      encryptedAccessToken: "1-nekot-ssecca",
      encryptedRefreshToken: "1-nekot-hserfer",
    });
  });

  it("returns null when no Dropbox account is connected", async () => {
    const dbPath = tempDatabasePath();
    await runSqliteMigrations(dbPath);

    await expect(
      new SqliteDropboxAccountRepository(dbPath, new ReversingCipher()).find(),
    ).resolves.toBeNull();
  });

  it("disconnects a Dropbox account", async () => {
    const dbPath = tempDatabasePath();
    await runSqliteMigrations(dbPath);
    const repository = new SqliteDropboxAccountRepository(
      dbPath,
      new ReversingCipher(),
    );

    await repository.save({
      accountId: "dbid:account-1",
      accessToken: "access-token-1",
      refreshToken: "refresh-token-1",
      accessTokenExpiresAtUnixSeconds: 1_780_000_000,
      connectedAtUnixSeconds: 1_770_000_000,
    });
    await repository.disconnect();

    await expect(repository.find()).resolves.toBeNull();
  });
});

class ReversingCipher implements DropboxTokenCipher {
  protect(token: string): string {
    return [...token].reverse().join("");
  }

  unprotect(encryptedToken: string): string {
    return [...encryptedToken].reverse().join("");
  }
}

async function storedTokenColumns(dbPath: string): Promise<{
  encryptedAccessToken: string;
  encryptedRefreshToken: string;
}> {
  const SQL = await initSqlJs();
  const database = new SQL.Database(readFileSync(dbPath));
  try {
    const result = database.exec(
      "SELECT encrypted_access_token, encrypted_refresh_token FROM dropbox_accounts",
    );
    return {
      encryptedAccessToken: String(result[0].values[0][0]),
      encryptedRefreshToken: String(result[0].values[0][1]),
    };
  } finally {
    database.close();
  }
}

function tempDatabasePath(): string {
  return join(tmpdir(), `muzo-${Date.now()}-${Math.random()}.sqlite`);
}
