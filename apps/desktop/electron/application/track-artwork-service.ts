import type { DropboxLibraryService } from "./dropbox-library-service";
import type { TrackArtworkSources, FileArtworkReader, ArtworkThumbnails, DropboxArtworkReader } from "./interfaces/track-artwork";
import { failure, success, type Result } from "./result";

export class TrackArtworkService {
  constructor(private readonly sources: TrackArtworkSources, private readonly files: FileArtworkReader, private readonly thumbnails: ArtworkThumbnails, private readonly remote: DropboxArtworkReader, private readonly accounts: Pick<DropboxLibraryService, "accessFor">) {}
  async get(libraryId: string, trackId: string): Promise<Result<string | null>> {
    try {
      const source = await this.sources.find(libraryId, trackId);
      if (!source) return failure("trackNotFound", "This track is no longer in the library.");
      const access = source.kind === "dropbox" ? await this.accounts.accessFor(libraryId) : null;
      if (access && !access.ok) return access;
      const picture = access?.ok
        ? await this.remote.read(access.value.accessToken, source.path)
        : await this.files.read(source.path);
      return success(picture ? this.thumbnails.create(picture) : null);
    } catch {
      return failure("repository", "Album artwork could not be read.");
    }
  }
}
