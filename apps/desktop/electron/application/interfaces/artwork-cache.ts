import type { Result } from "../result";

export interface CachedArtwork {
  uri: string | null;
}

export interface ArtworkCache {
  find(key: string): Promise<Result<CachedArtwork | null>>;
  save(key: string, uri: string | null): Promise<Result<void>>;
}
