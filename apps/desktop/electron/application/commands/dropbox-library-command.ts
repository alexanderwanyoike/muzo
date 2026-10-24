import { failure, type Result } from "../../../shared/result";
import type { DropboxLibraryService } from "../dropbox-library-service";
import type { DropboxSyncController } from "../interfaces/dropbox-sync-controller";
import type { CommandHandler } from "./command-handler";

export type DropboxLibraryCommandName =
  | "dropbox_add_library"
  | "dropbox_sync_library"
  | "dropbox_sync_status"
  | "dropbox_set_sync_interval";

export class DropboxLibraryCommand implements CommandHandler {
  constructor(
    readonly command: DropboxLibraryCommandName,
    private readonly libraries: DropboxLibraryService,
    private readonly sync: DropboxSyncController,
  ) {}

  async handle(args: unknown): Promise<Result<unknown>> {
    if (this.command === "dropbox_sync_status") return this.sync.status();
    const input =
      args && typeof args === "object" && "input" in args ? args.input : null;
    if (!input || typeof input !== "object")
      return failure("invalidInput", "Missing Dropbox input.");
    const values = input as Record<string, unknown>;
    if (this.command === "dropbox_set_sync_interval") {
      return typeof values.minutes === "number"
        ? this.sync.setIntervalMinutes(values.minutes)
        : failure("invalidInput", "Enter a sync interval in minutes.");
    }
    if (
      this.command === "dropbox_add_library" &&
      values.accountId !== undefined &&
      (typeof values.accountId !== "string" || !values.accountId)
    )
      return failure(
        "invalidInput",
        "Choose a Dropbox folder from your connected account.",
      );
    if (
      this.command === "dropbox_add_library" &&
      typeof values.name === "string" &&
      typeof values.location === "string"
    ) {
      return this.libraries.add({
        name: values.name,
        location: values.location,
        accountId:
          typeof values.accountId === "string" ? values.accountId : undefined,
      });
    }
    if (
      this.command === "dropbox_sync_library" &&
      typeof values.libraryId === "string" &&
      values.libraryId.trim()
    ) {
      return this.libraries.sync(values.libraryId);
    }
    return failure("invalidInput", "Enter valid Dropbox library details.");
  }
}
