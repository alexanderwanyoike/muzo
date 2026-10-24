import type { BrowseDropboxFoldersService } from "../browse-dropbox-folders-service";
import { failure, type Result } from "../result";
import type { CommandHandler } from "./command-handler";

export class BrowseDropboxFoldersCommand implements CommandHandler {
  readonly command = "dropbox_list_folders";

  constructor(private readonly folders: BrowseDropboxFoldersService) {}

  async handle(args: unknown): Promise<Result<unknown>> {
    const input =
      args && typeof args === "object" && "input" in args ? args.input : null;
    if (
      !input ||
      typeof input !== "object" ||
      !("accountId" in input) ||
      typeof input.accountId !== "string" ||
      !input.accountId ||
      !("path" in input) ||
      typeof input.path !== "string" ||
      !input.path.startsWith("/")
    ) {
      return failure(
        "invalidInput",
        "Choose a Dropbox folder from your connected account.",
      );
    }
    return this.folders.browse(input.accountId, input.path);
  }
}
