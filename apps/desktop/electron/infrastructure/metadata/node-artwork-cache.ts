import { createHash, randomUUID } from "node:crypto";
import {
  mkdir,
  open,
  readdir,
  rename,
  rm,
  stat,
  utimes,
  writeFile,
} from "node:fs/promises";
import { join } from "node:path";
import type {
  ArtworkCache,
  CachedArtwork,
} from "../../application/interfaces/artwork-cache";
import { failure, success, type Result } from "../../application/result";

const maximumUriLength = 256 * 1024;
const maximumRecordBytes = maximumUriLength + 128;
const pngPrefix = "data:image/png;base64,";
const pngSignature = Buffer.from("89504e470d0a1a0a", "hex");
function validUri(uri: string | null): boolean {
  return (
    uri === null ||
    (uri.length <= maximumUriLength &&
      uri.startsWith(pngPrefix) &&
      Buffer.from(uri.slice(pngPrefix.length), "base64")
        .subarray(0, pngSignature.length)
        .equals(pngSignature))
  );
}

const maximumBytes = 128 * 1024 * 1024;
const maximumEntries = 4096;
function hash(value: string): string {
  return createHash("sha256").update(value).digest("hex");
}

const activeWrites = new Set<string>();
const temporaryFilename = /^(?:(\d+)-)?[a-f0-9]{8}-[a-f0-9]{4}-[a-f0-9]{4}-[a-f0-9]{4}-[a-f0-9]{12}\.tmp$/;
const cacheFilename = /^[a-f0-9]{64}\.json$/;
interface Limits {
  maximumBytes?: number;
  maximumEntries?: number;
}

export class NodeArtworkCache implements ArtworkCache {
  private entries: Map<string, number> | null = null;

  private working: Promise<void> = Promise.resolve();

  constructor(
    private readonly directory: string,
    private readonly limits: Limits = {},
  ) {}

  find(key: string): Promise<Result<CachedArtwork | null>> {
    return this.exclusive(async () => {
      const filename = this.filename(key);
      try {
        await this.loadIndex();
        const path = join(this.directory, filename);
        const text = await this.readBounded(path);
        if (text === null) return await this.discard(filename);
        const value: unknown = JSON.parse(text);
        if (
          value &&
          typeof value === "object" &&
          "uri" in value &&
          (value.uri === null ||
            (typeof value.uri === "string" &&
              value.uri.startsWith("data:image/png;base64,"))) &&
          "digest" in value &&
          value.digest === hash(JSON.stringify(value.uri)) &&
          validUri(value.uri)
        ) {
          this.touch(filename, Buffer.byteLength(text));
          const now = new Date();
          await utimes(path, now, now);
          await this.trim();
          return success({ uri: value.uri });
        }
        return await this.discard(filename);
      } catch (error) {
        if (error instanceof SyntaxError) return this.discard(filename);
        if (
          error &&
          typeof error === "object" &&
          "code" in error &&
          error.code === "ENOENT"
        ) {
          this.entries?.delete(filename);
          return success(null);
        }
        return failure("storage", "Artwork cache could not be read.");
      }
    });
  }

  save(key: string, uri: string | null): Promise<Result<void>> {
    if (!validUri(uri))
      return Promise.resolve(
        failure("invalidInput", "Artwork thumbnail is invalid or too large."),
      );
    return this.exclusive(async () => {
      const temporary = join(this.directory, `${process.pid}-${randomUUID()}.tmp`);
      activeWrites.add(temporary);
      try {
        await this.loadIndex();
        const text = JSON.stringify({ uri, digest: hash(JSON.stringify(uri)) });
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
        activeWrites.delete(temporary);
      }
    });
  }

  private async readBounded(path: string): Promise<string | null> {
    const file = await open(path, "r");
    try {
      const { size } = await file.stat();
      if (size > maximumRecordBytes) return null;
      const buffer = Buffer.alloc(size + 1);
      let offset = 0;
      while (offset < buffer.length) {
        const { bytesRead } = await file.read(
          buffer,
          offset,
          buffer.length - offset,
          offset,
        );
        if (bytesRead === 0) break;
        offset += bytesRead;
      }
      return offset > size ? null : buffer.subarray(0, offset).toString("utf8");
    } finally {
      await file.close();
    }
  }

  private async discard(filename: string): Promise<Result<null>> {
    try {
      await rm(join(this.directory, filename), { force: true });
      this.entries?.delete(filename);
      return success(null);
    } catch {
      return failure("storage", "Invalid artwork cache could not be removed.");
    }
  }

  private async loadIndex(): Promise<void> {
    if (this.entries) return;
    await mkdir(this.directory, { recursive: true, mode: 0o700 });
    const files = await readdir(this.directory, { withFileTypes: true });
    const entries: { name: string; bytes: number; modified: number }[] = [];
    for (const file of files) {
      if (!file.isFile()) continue;
      if (temporaryFilename.test(file.name)) {
        await this.removeAbandonedWrite(file.name);
        continue;
      }
      if (!cacheFilename.test(file.name)) continue;
      const info = await stat(join(this.directory, file.name));
      entries.push({
        name: file.name,
        bytes: info.size,
        modified: info.mtimeMs,
      });
    }
    entries.sort((a, b) => a.modified - b.modified);
    this.entries = new Map(entries.map((entry) => [entry.name, entry.bytes]));
    await this.trim();
  }

  private async removeAbandonedWrite(filename: string): Promise<void> {
    const path = join(this.directory, filename);
    if (activeWrites.has(path)) return;
    const owner = Number(temporaryFilename.exec(filename)?.[1] ?? 0);
    if (owner > 0 && owner !== process.pid) {
      try {
        process.kill(owner, 0);
        return;
      } catch (error) {
        if (!error || typeof error !== "object" || !("code" in error) || error.code !== "ESRCH") return;
      }
    }
    await rm(path, { force: true });
  }

  private touch(filename: string, bytes: number): void {
    this.entries?.delete(filename);
    this.entries?.set(filename, bytes);
  }

  private async trim(): Promise<void> {
    if (!this.entries) return;
    let bytes = [...this.entries.values()].reduce((sum, size) => sum + size, 0);
    while (
      this.entries.size > (this.limits.maximumEntries ?? maximumEntries) ||
      bytes > (this.limits.maximumBytes ?? maximumBytes)
    ) {
      const oldest = this.entries.entries().next().value;
      if (!oldest) break;
      await rm(join(this.directory, oldest[0]), { force: true });
      this.entries.delete(oldest[0]);
      bytes -= oldest[1];
    }
  }

  private exclusive<T>(
    operation: () => Promise<Result<T>>,
  ): Promise<Result<T>> {
    const result = this.working.then(operation);
    this.working = result.then(
      () => undefined,
      () => undefined,
    );
    return result;
  }

  private filename(key: string): string {
    return `${hash(key)}.json`;
  }
}
