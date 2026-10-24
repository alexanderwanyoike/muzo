import type {
  DropboxConnectionStatus,
  DropboxSyncStatus,
} from "../shared/dropbox-contracts";
import type { LibraryDto } from "./types";
import type { ScanReport } from "./api";
import { failure, type Result } from "../shared/result";
import { invokeCommand } from "./runtime-bridge";

export async function dropboxCommand<T>(
  command: string,
  input?: unknown,
): Promise<Result<T>> {
  try {
    return await invokeCommand<Result<T>>(
      command,
      input === undefined ? undefined : { input },
    );
  } catch {
    return failure(
      "remote",
      "Dropbox is unavailable. Check the connection and try again.",
    );
  }
}

export const dropboxStatus = () =>
  dropboxCommand<DropboxConnectionStatus>("dropbox_status");
export const disconnectDropbox = () =>
  dropboxCommand<DropboxConnectionStatus>("dropbox_disconnect");
export const beginDropboxConnection = (clientId: string) =>
  dropboxCommand<{ pending: true }>("dropbox_begin", { clientId });
export const completeDropboxConnection = (code: string) =>
  dropboxCommand<DropboxConnectionStatus>("dropbox_complete", { code });
export const addDropboxLibrary = (input: { name: string; location: string }) =>
  dropboxCommand<LibraryDto>("dropbox_add_library", input);
export const syncDropboxLibrary = (libraryId: string) =>
  dropboxCommand<ScanReport>("dropbox_sync_library", { libraryId });
export const dropboxSyncStatus = () =>
  dropboxCommand<DropboxSyncStatus>("dropbox_sync_status");
export const setDropboxSyncInterval = (minutes: number) =>
  dropboxCommand<{ intervalMinutes: number }>("dropbox_set_sync_interval", {
    minutes,
  });
