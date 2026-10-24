import { randomUUID } from "node:crypto";
// @vitest-environment node
import {
  mkdtempSync,
  rmSync,
  readdirSync,
  statSync,
  readFileSync,
  writeFileSync,
} from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { expect, it } from "vitest";
import { NodeArtworkCache } from "./node-artwork-cache";

it("discards an oversized cache record instead of loading it", async () => {
  const directory = mkdtempSync(join(tmpdir(), "muzo-cache-oversized-"));
  try {
    const cache = new NodeArtworkCache(directory);
    await cache.save("song", null);
    const file = join(directory, readdirSync(directory)[0]);
    writeFileSync(
      file,
      `${readFileSync(file, "utf8")}${" ".repeat(300 * 1024)}`,
    );
    expect(await cache.find("song")).toEqual({ ok: true, value: null });
  } finally {
    rmSync(directory, { recursive: true, force: true });
  }
});

it.each([
  "data:image/svg+xml,<svg/>",
  `data:image/png;base64,${"A".repeat(256 * 1024)}`,
])("refuses an invalid or oversized thumbnail", async (uri) => {
  const directory = mkdtempSync(join(tmpdir(), "muzo-cache-invalid-"));
  try {
    expect(
      await new NodeArtworkCache(directory).save("song", uri),
    ).toMatchObject({ ok: false, error: { kind: "invalidInput" } });
  } finally {
    rmSync(directory, { recursive: true, force: true });
  }
});

it("discards a thumbnail whose stored image bytes were corrupted", async () => {
  const directory = mkdtempSync(join(tmpdir(), "muzo-cache-corrupt-"));
  try {
    const uri = `data:image/png;base64,${readFileSync(new URL("./fixtures/cover.png", import.meta.url)).toString("base64")}`;
    const cache = new NodeArtworkCache(directory);
    await cache.save("song", uri);
    const file = join(directory, readdirSync(directory)[0]);
    const record = JSON.parse(readFileSync(file, "utf8")) as { uri: string };
    record.uri = `${record.uri.slice(0, 100)}${record.uri[100] === "A" ? "B" : "A"}${record.uri.slice(101)}`;
    writeFileSync(file, JSON.stringify(record));
    expect(await new NodeArtworkCache(directory).find("song")).toEqual({
      ok: true,
      value: null,
    });
  } finally {
    rmSync(directory, { recursive: true, force: true });
  }
});

it.each(["entries", "bytes"])(
  "evicts the least recently used thumbnail to respect the %s budget",
  async (budget) => {
    const directory = mkdtempSync(join(tmpdir(), "muzo-cache-budget-"));
    try {
      await new NodeArtworkCache(directory).save("seed", null);
      const entryBytes = readdirSync(directory).reduce(
        (sum, file) => sum + statSync(join(directory, file)).size,
        0,
      );
      rmSync(directory, { recursive: true });
      const limits =
        budget === "entries"
          ? { maximumEntries: 2, maximumBytes: entryBytes * 100 }
          : { maximumEntries: 100, maximumBytes: entryBytes * 2 };
      const cache = new NodeArtworkCache(directory, limits);
      await cache.save("a", null);
      await cache.save("b", null);
      await cache.find("a");
      await cache.save("c", null);
      expect(await cache.find("b")).toEqual({ ok: true, value: null });
      expect(await new NodeArtworkCache(directory, limits).find("a")).toEqual({
        ok: true,
        value: { uri: null },
      });
      expect(await cache.find("c")).toEqual({ ok: true, value: { uri: null } });
      const sizes = readdirSync(directory).map(
        (file) => statSync(join(directory, file)).size,
      );
      expect(sizes.length).toBeLessThanOrEqual(limits.maximumEntries);
      expect(sizes.reduce((sum, size) => sum + size, 0)).toBeLessThanOrEqual(
        limits.maximumBytes,
      );
    } finally {
      rmSync(directory, { recursive: true, force: true });
    }
  },
);

it("replaces a truncated record and stores private cache files", async () => {
  const directory = mkdtempSync(join(tmpdir(), "muzo-cache-private-"));
  const cacheDir = join(directory, "artwork");
  try {
    const cache = new NodeArtworkCache(cacheDir);
    await cache.save("../../untrusted-source", null);
    const files = readdirSync(cacheDir);
    expect(files).toHaveLength(1);
    const file = join(cacheDir, files[0]);
    if (process.platform !== "win32") {
      expect(statSync(cacheDir).mode % 0o1000).toBe(0o700);
      expect(statSync(file).mode % 0o1000).toBe(0o600);
    }
    writeFileSync(file, "{");
    expect(await cache.find("../../untrusted-source")).toEqual({
      ok: true,
      value: null,
    });
    expect(await cache.save("../../untrusted-source", null)).toEqual({
      ok: true,
      value: undefined,
    });
    expect(await cache.find("../../untrusted-source")).toEqual({
      ok: true,
      value: { uri: null },
    });
  } finally {
    rmSync(directory, { recursive: true, force: true });
  }
});

it("serializes concurrent cache writes and reports an unavailable cache directory", async () => {
  const directory = mkdtempSync(join(tmpdir(), "muzo-cache-concurrent-"));
  try {
    const cache = new NodeArtworkCache(join(directory, "valid"));
    const results = await Promise.all(
      Array.from({ length: 12 }, (_, i) => cache.save(String(i), null)),
    );
    expect(results.every((result) => result.ok)).toBe(true);
    expect(await cache.find("11")).toEqual({ ok: true, value: { uri: null } });
    const unavailable = join(directory, "file");
    writeFileSync(unavailable, "not a directory");
    expect(
      await new NodeArtworkCache(unavailable).save("song", null),
    ).toMatchObject({ ok: false, error: { kind: "storage" } });
  } finally {
    rmSync(directory, { recursive: true, force: true });
  }
});


it.each(["legacy", "current-process", "exited-process"])("removes an abandoned %s temporary thumbnail before enforcing the cache budget", async (kind) => {
  const directory = mkdtempSync(join(tmpdir(), "muzo-cache-abandoned-"));
  const absentProcessId = 2147483647;
  const prefix = kind === "legacy" ? "" : `${kind === "current-process" ? process.pid : absentProcessId}-`;
  try {
    writeFileSync(join(directory, `${prefix}${randomUUID()}.tmp`), "x".repeat(1024));
    const cache = new NodeArtworkCache(directory, { maximumBytes: 100, maximumEntries: 1 });
    expect(await cache.save("song", null)).toEqual({ ok: true, value: undefined });
    const files = readdirSync(directory);
    expect(files).toHaveLength(1);
    expect(files.every((file) => file.endsWith(".json"))).toBe(true);
    expect(files.reduce((sum, file) => sum + statSync(join(directory, file)).size, 0)).toBeLessThanOrEqual(100);
  } finally { rmSync(directory, { recursive: true, force: true }); }
});
