// @vitest-environment node
import { mkdtempSync, rmSync, readdirSync, statSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { expect, it } from "vitest";
import { NodeArtworkCache } from "./node-artwork-cache";

it.each(["entries", "bytes"])("evicts the least recently used thumbnail to respect the %s budget", async (budget) => {
  const directory = mkdtempSync(join(tmpdir(), "muzo-cache-budget-"));
  try {
    await new NodeArtworkCache(directory).save("seed", null);
    const entryBytes = readdirSync(directory).reduce((sum, file) => sum + statSync(join(directory, file)).size, 0);
    rmSync(directory, { recursive: true });
    const limits = budget === "entries" ? { maximumEntries: 2, maximumBytes: entryBytes * 100 } : { maximumEntries: 100, maximumBytes: entryBytes * 2 };
    const cache = new NodeArtworkCache(directory, limits);
    await cache.save("a", null);
    await cache.save("b", null);
    await cache.find("a");
    await cache.save("c", null);
    expect(await cache.find("b")).toEqual({ ok: true, value: null });
    expect(await new NodeArtworkCache(directory, limits).find("a")).toEqual({ ok: true, value: { uri: null } });
    expect(await cache.find("c")).toEqual({ ok: true, value: { uri: null } });
    const sizes = readdirSync(directory).map((file) => statSync(join(directory, file)).size);
    expect(sizes.length).toBeLessThanOrEqual(limits.maximumEntries);
    expect(sizes.reduce((sum, size) => sum + size, 0)).toBeLessThanOrEqual(limits.maximumBytes);
  } finally { rmSync(directory, { recursive: true, force: true }); }
});
