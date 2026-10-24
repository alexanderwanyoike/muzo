import type { DropboxTokenCipher } from "../../application/interfaces/dropbox-account-interfaces";

export interface ElectronSafeStorage {
  encryptString(value: string): Buffer;
  decryptString(value: Buffer): string;
}

export class ElectronSafeStorageDropboxTokenCipher
  implements DropboxTokenCipher
{
  constructor(private readonly safeStorage: ElectronSafeStorage) {}

  protect(token: string): string {
    return this.safeStorage.encryptString(token).toString("base64");
  }

  unprotect(encryptedToken: string): string {
    return this.safeStorage.decryptString(Buffer.from(encryptedToken, "base64"));
  }
}
