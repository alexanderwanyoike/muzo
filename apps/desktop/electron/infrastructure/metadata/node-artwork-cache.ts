import { createHash, randomUUID } from "node:crypto";
import { mkdir, readFile, rename, rm, writeFile } from "node:fs/promises";
import { join } from "node:path";
import type { ArtworkCache, CachedArtwork } from "../../application/interfaces/artwork-cache";
import { failure, success, type Result } from "../../application/result";

export class NodeArtworkCache implements ArtworkCache {
  constructor(private readonly directory: string) {}

  async find(key: string): Promise<Result<CachedArtwork | null>> {
    try {
      const value: unknown = JSON.parse(await readFile(this.path(key), "utf8"));
      if (value && typeof value === "object" && "uri" in value && (value.uri === null || (typeof value.uri === "string" && value.uri.startsWith("data:image/png;base64,")))) return success({ uri: value.uri });
      return success(null);
    } catch (error) {
      if (error && typeof error === "object" && "code" in error && error.code === "ENOENT") return success(null);
      return failure("storage", "Artwork cache could not be read.");
    }
  }

  async save(key: string, uri: string | null): Promise<Result<void>> {
    const temporary = join(this.directory, `${randomUUID()}.tmp`);
    try {
      await mkdir(this.directory, { recursive: true, mode: 0o700 });
      await writeFile(temporary, JSON.stringify({ uri }), { mode: 0o600 });
      await rename(temporary, this.path(key));
      return success(undefined);
    } catch {
      return failure("storage", "Artwork cache could not be saved.");
    } finally {
      await rm(temporary, { force: true }).catch(() => undefined);
    }
  }

  private path(key: string): string {
    return join(this.directory, `${createHash("sha256").update(key).digest("hex")}.json`);
  }
}
