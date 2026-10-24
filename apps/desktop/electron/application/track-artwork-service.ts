import type { DropboxLibraryService } from "./dropbox-library-service";
import type {
  TrackArtworkSources,
  TrackArtworkSource,
  FileArtworkReader,
  ArtworkThumbnails,
  DropboxArtworkReader,
} from "./interfaces/track-artwork";
import { failure, success, type Result } from "./result";

const maximumCachedCovers = 128;

export class TrackArtworkService {
  private readonly cache = new Map<string, string | null>();
  private readonly inFlight = new Map<string, Promise<Result<string | null>>>();
  private reading: Promise<void> = Promise.resolve();

  constructor(
    private readonly sources: TrackArtworkSources,
    private readonly files: FileArtworkReader,
    private readonly thumbnails: ArtworkThumbnails,
    private readonly remote: DropboxArtworkReader,
    private readonly accounts: Pick<DropboxLibraryService, "accessFor">,
  ) {}

  async get(
    libraryId: string,
    trackId: string,
  ): Promise<Result<string | null>> {
    try {
      const source = await this.sources.find(libraryId, trackId);
      if (!source)
        return failure(
          "trackNotFound",
          "This track is no longer in the library.",
        );
      const access =
        source.kind === "dropbox"
          ? await this.accounts.accessFor(libraryId)
          : null;
      if (access && !access.ok) return access;
      const key = JSON.stringify([
        libraryId,
        trackId,
        source.path,
        source.revision,
      ]);
      if (this.cache.has(key)) return success(this.cache.get(key) ?? null);
      const existing = this.inFlight.get(key);
      if (existing) return existing;
      const pending = this.reading
        .then(() =>
          this.read(source, access?.ok ? access.value.accessToken : null),
        )
        .then((result) => {
          if (result.ok) {
            this.cache.set(key, result.value);
            const oldest = this.cache.keys().next().value;
            if (this.cache.size > maximumCachedCovers && oldest !== undefined)
              this.cache.delete(oldest);
          }
          return result;
        })
        .finally(() => {
          this.inFlight.delete(key);
        });
      this.inFlight.set(key, pending);
      this.reading = pending.then(() => undefined);
      return pending;
    } catch {
      return failure("repository", "Album artwork could not be read.");
    }
  }

  private async read(
    source: TrackArtworkSource,
    accessToken: string | null,
  ): Promise<Result<string | null>> {
    try {
      const picture = accessToken
        ? await this.remote.read(accessToken, source.path)
        : await this.files.read(source.path);
      return success(picture ? this.thumbnails.create(picture) : null);
    } catch {
      return failure(
        source.kind === "dropbox" ? "remote" : "repository",
        "Album artwork could not be read.",
      );
    }
  }
}
