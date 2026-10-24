import type { DropboxLibraryAccess } from "./dropbox-library-service";
import type {
  DropboxFolderListing,
  DropboxFolderReader,
} from "./interfaces/dropbox-folder-interfaces";
import type { DropboxLibrarySettings } from "./interfaces/dropbox-library-settings";
import type { LibraryRepository } from "./interfaces/repository-interfaces";
import { failure, success, type Result } from "./result";

export class BrowseDropboxFoldersService {
  constructor(
    private readonly account: Pick<DropboxLibraryAccess, "accessToken">,
    private readonly folders: DropboxFolderReader,
    private readonly libraries: LibraryRepository,
    private readonly settings: Pick<DropboxLibrarySettings, "accountFor">,
  ) {}

  async browse(
    accountId: string,
    path: string,
  ): Promise<Result<DropboxFolderListing>> {
    const token = await this.account.accessToken(accountId);
    if (!token.ok) return token;
    try {
      const folders = await this.folders.listFolders(token.value, path);
      const addedPaths: string[] = [];
      for (const library of await this.libraries.list()) {
        if (
          library.kind === "dropbox" &&
          (await this.settings.accountFor(library.id)) === accountId
        )
          addedPaths.push(library.location);
      }
      return success({ folders, addedPaths });
    } catch {
      return failure(
        "remote",
        "Could not load Dropbox folders. Check your connection and try again.",
      );
    }
  }
}
