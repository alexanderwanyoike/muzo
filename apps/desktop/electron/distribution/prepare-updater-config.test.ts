// @vitest-environment node
import { mkdtemp, readFile, rm } from "node:fs/promises";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { expect, it } from "vitest";
import { prepareUpdaterConfig } from "./prepare-updater-config";

it("provides the updater cache and fixed feed configuration inside prepackaged resources", async () => {
  const directory = await mkdtemp(join(tmpdir(), "muzo-prepackaged-update-"));
  try {
    expect((await prepareUpdaterConfig(directory)).ok).toBe(true);
    const config = await readFile(
      join(directory, "resources", "app-update.yml"),
      "utf8",
    );
    expect(config).toContain("updaterCacheDirName: muzo-updater");
    expect(config).toContain("provider: github");
    expect(config).toContain("owner: alexanderwanyoike");
    expect(config).toContain("repo: muzo");
  } finally {
    await rm(directory, { recursive: true, force: true });
  }
});
