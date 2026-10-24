import type { DropboxTokenCipher } from "../../application/interfaces/dropbox-account-interfaces";

export interface ElectronSafeStorage {
  isEncryptionAvailable(): boolean;
  getSelectedStorageBackend?(): string;
  encryptString(value: string): Buffer;
  decryptString(value: Buffer): string;
}

export class ElectronSafeStorageDropboxTokenCipher implements DropboxTokenCipher {
  constructor(private readonly safeStorage: ElectronSafeStorage) {}

  protect(token: string): string {
    if (
      !this.safeStorage.isEncryptionAvailable() ||
      this.safeStorage.getSelectedStorageBackend?.() === "basic_text"
    ) {
      throw new Error(
        "OS secure storage is unavailable. Enable a system keyring and try again.",
      );
    }
    return this.safeStorage.encryptString(token).toString("base64");
  }

  unprotect(encryptedToken: string): string {
    return this.safeStorage.decryptString(
      Buffer.from(encryptedToken, "base64"),
    );
  }
}
