export interface StoredDropboxAccount {
  accountId: string;
  accessToken: string;
  refreshToken: string;
  accessTokenExpiresAtUnixSeconds: number;
  connectedAtUnixSeconds: number;
}

export interface DropboxAccountRepository {
  disconnect(): Promise<void>;
  find(): Promise<StoredDropboxAccount | null>;
  save(account: StoredDropboxAccount): Promise<void>;
}

export interface DropboxTokenCipher {
  protect(token: string): string;
  unprotect(encryptedToken: string): string;
}
