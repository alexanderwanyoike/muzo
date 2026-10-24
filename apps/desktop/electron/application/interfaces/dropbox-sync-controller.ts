import type { DropboxSyncStatus } from "../../../shared/dropbox-contracts";
import type { Result } from "../../../shared/result";

export interface DropboxSyncController {
  status(): Promise<Result<DropboxSyncStatus>>;
  setIntervalMinutes(minutes: number): Promise<Result<{ intervalMinutes: number }>>;
}
