export interface DropboxConnectionStatus {
  connected: boolean;
  accountId: string | null;
}

export interface DropboxSyncStatus {
  intervalMinutes: number;
  syncing: boolean;
  lastError: string | null;
  lastSyncedAtUnixSeconds: number | null;
}
