import type { TrackArtworkSources, FileArtworkReader, ArtworkThumbnails } from "./interfaces/track-artwork";
import { failure, success, type Result } from "./result";

export class TrackArtworkService {
  constructor(private readonly sources: TrackArtworkSources, private readonly files: FileArtworkReader, private readonly thumbnails: ArtworkThumbnails) {}
  async get(libraryId: string, trackId: string): Promise<Result<string | null>> {
    try {
      const source = await this.sources.find(libraryId, trackId);
      if (!source) return failure("trackNotFound", "This track is no longer in the library.");
      if (source.kind !== "filesystem") return failure("remote", "Dropbox artwork could not be read.");
      const picture = await this.files.read(source.path);
      return success(picture ? this.thumbnails.create(picture) : null);
    } catch {
      return failure("repository", "Album artwork could not be read.");
    }
  }
}
