import type { DropboxConnectionStatus } from "../shared/dropbox-contracts";
import { failure, type Result } from "../shared/result";
import { invokeCommand } from "./runtime-bridge";

export async function dropboxCommand<T>(command: string, input?: unknown): Promise<Result<T>> {
  try {
    return await invokeCommand<Result<T>>(command, input === undefined ? undefined : { input });
  } catch {
    return failure("remote", "Dropbox is unavailable. Check the connection and try again.");
  }
}

export const dropboxStatus = () => dropboxCommand<DropboxConnectionStatus>("dropbox_status");
export const disconnectDropbox = () => dropboxCommand<DropboxConnectionStatus>("dropbox_disconnect");
export const beginDropboxConnection = (clientId: string) => dropboxCommand<{ pending: true }>("dropbox_begin", { clientId });
export const completeDropboxConnection = (code: string) => dropboxCommand<DropboxConnectionStatus>("dropbox_complete", { code });
