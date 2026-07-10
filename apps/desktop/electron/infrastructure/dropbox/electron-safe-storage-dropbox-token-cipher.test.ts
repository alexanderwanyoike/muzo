// @vitest-environment node

import { describe, expect, it } from "vitest";

import { ElectronSafeStorageDropboxTokenCipher } from "./electron-safe-storage-dropbox-token-cipher";

describe("ElectronSafeStorageDropboxTokenCipher", () => {
  it("protects tokens as encoded ciphertext and restores plaintext", () => {
    const safeStorage = new FakeSafeStorage();
    const cipher = new ElectronSafeStorageDropboxTokenCipher(safeStorage);

    const encrypted = cipher.protect("refresh-token-1");

    expect(encrypted).toBe("ZW5jOnJlZnJlc2gtdG9rZW4tMQ==");
    expect(encrypted).not.toContain("refresh-token-1");
    expect(cipher.unprotect(encrypted)).toBe("refresh-token-1");
  });
});

class FakeSafeStorage {
  encryptString(value: string): Buffer {
    return Buffer.from(`enc:${value}`);
  }

  decryptString(value: Buffer): string {
    return value.toString("utf8").replace(/^enc:/, "");
  }
}
