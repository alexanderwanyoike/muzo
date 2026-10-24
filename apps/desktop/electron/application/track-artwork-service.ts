import type { ArtworkCache } from "./interfaces/artwork-cache";
import type { Logger } from "./interfaces/logger-interfaces";
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
    private readonly persisted: ArtworkCache,
    private readonly logger: Logger,
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
      const key = JSON.stringify([
        libraryId,
        trackId,
        source.path,
        source.revision,
      ]);
      if (this.cache.has(key)) return success(this.cache.get(key) ?? null);
      const existing = this.inFlight.get(key);
      if (existing) return await existing;
      const pending = this.reading
        .then(async () => {
          const cached = await this.persisted.find(key);
          if (cached.ok && cached.value) return success(cached.value.uri);
          if (!cached.ok) this.logger.error(cached.error.message);
          const access =
            source.kind === "dropbox"
              ? await this.accounts.accessFor(libraryId)
              : null;
          if (access && !access.ok) return access;
          const result = await this.read(
            source,
            access?.ok ? access.value.accessToken : null,
          );
          if (result.ok) {
            const saved = await this.persisted.save(key, result.value);
            if (!saved.ok) this.logger.error(saved.error.message);
          }
          return result;
        })
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
      this.reading = pending.then(
        () => undefined,
        () => undefined,
      );
      return await pending;
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
