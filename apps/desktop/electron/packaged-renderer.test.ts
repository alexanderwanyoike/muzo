// @vitest-environment node
import { existsSync, mkdtempSync, readFileSync, rmSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";
import { build } from "vite";
import { expect, it } from "vitest";

it("loads every renderer asset relative to the packaged index file", async () => {
  const output = mkdtempSync(join(tmpdir(), "muzo-packaged-renderer-"));
  try {
    await build({ configFile: fileURLToPath(new URL("../vite.config.ts", import.meta.url)), root: fileURLToPath(new URL("..", import.meta.url)), logLevel: "silent", build: { outDir: output, emptyOutDir: true } });
    const index = join(output, "index.html");
    const paths = [...readFileSync(index, "utf8").matchAll(/(?:src|href)="([^"]+)"/g)].map((match) => match[1]);
    expect(paths.length).toBeGreaterThan(0);
    for (const path of paths) {
      expect(existsSync(fileURLToPath(new URL(path, pathToFileURL(index)))), path).toBe(true);
    }
  } finally { rmSync(output, { recursive: true, force: true }); }
});
