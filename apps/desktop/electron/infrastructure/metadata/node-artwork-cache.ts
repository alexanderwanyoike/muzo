import { createHash, randomUUID } from "node:crypto";
import { mkdir, readFile, readdir, rename, rm, stat, utimes, writeFile } from "node:fs/promises";
import { join } from "node:path";
import type { ArtworkCache, CachedArtwork } from "../../application/interfaces/artwork-cache";
import { failure, success, type Result } from "../../application/result";

const maximumBytes = 128 * 1024 * 1024;
const maximumEntries = 4096;
const cacheFilename = /^[a-f0-9]{64}\.json$/;
interface Limits { maximumBytes?: number; maximumEntries?: number; }

export class NodeArtworkCache implements ArtworkCache {
  private entries: Map<string, number> | null = null;

  private working: Promise<void> = Promise.resolve();

  constructor(private readonly directory: string, private readonly limits: Limits = {}) {}

  find(key: string): Promise<Result<CachedArtwork | null>> {
    return this.exclusive(async () => {
      const filename = this.filename(key);
      try {
        await this.loadIndex();
        const path = join(this.directory, filename);
        const text = await readFile(path, "utf8");
        const value: unknown = JSON.parse(text);
        if (value && typeof value === "object" && "uri" in value && (value.uri === null || (typeof value.uri === "string" && value.uri.startsWith("data:image/png;base64,")))) {
          this.touch(filename, Buffer.byteLength(text));
          const now = new Date();
          await utimes(path, now, now);
          await this.trim();
          return success({ uri: value.uri });
        }
        return success(null);
      } catch (error) {
        if (error && typeof error === "object" && "code" in error && error.code === "ENOENT") {
          this.entries?.delete(filename);
          return success(null);
        }
        return failure("storage", "Artwork cache could not be read.");
      }
    });
  }

  save(key: string, uri: string | null): Promise<Result<void>> {
    return this.exclusive(async () => {
      const temporary = join(this.directory, `${randomUUID()}.tmp`);
      try {
        await this.loadIndex();
        const text = JSON.stringify({ uri });
        await writeFile(temporary, text, { mode: 0o600 });
        const filename = this.filename(key);
        await rename(temporary, join(this.directory, filename));
        this.touch(filename, Buffer.byteLength(text));
        await this.trim();
        return success(undefined);
      } catch {
        return failure("storage", "Artwork cache could not be saved.");
      } finally {
        await rm(temporary, { force: true }).catch(() => undefined);
      }
    });
  }

  private async loadIndex(): Promise<void> {
    if (this.entries) return;
    await mkdir(this.directory, { recursive: true, mode: 0o700 });
    const files = await readdir(this.directory, { withFileTypes: true });
    const entries: { name: string; bytes: number; modified: number }[] = [];
    for (const file of files) {
      if (!file.isFile() || !cacheFilename.test(file.name)) continue;
      const info = await stat(join(this.directory, file.name));
      entries.push({ name: file.name, bytes: info.size, modified: info.mtimeMs });
    }
    entries.sort((a, b) => a.modified - b.modified);
    this.entries = new Map(entries.map((entry) => [entry.name, entry.bytes]));
    await this.trim();
  }

  private touch(filename: string, bytes: number): void {
    this.entries?.delete(filename);
    this.entries?.set(filename, bytes);
  }

  private async trim(): Promise<void> {
    if (!this.entries) return;
    let bytes = [...this.entries.values()].reduce((sum, size) => sum + size, 0);
    while (this.entries.size > (this.limits.maximumEntries ?? maximumEntries) || bytes > (this.limits.maximumBytes ?? maximumBytes)) {
      const oldest = this.entries.entries().next().value;
      if (!oldest) break;
      await rm(join(this.directory, oldest[0]), { force: true });
      this.entries.delete(oldest[0]);
      bytes -= oldest[1];
    }
  }

  private exclusive<T>(operation: () => Promise<Result<T>>): Promise<Result<T>> {
    const result = this.working.then(operation);
    this.working = result.then(() => undefined, () => undefined);
    return result;
  }

  private filename(key: string): string {
    return `${createHash("sha256").update(key).digest("hex")}.json`;
  }
}
