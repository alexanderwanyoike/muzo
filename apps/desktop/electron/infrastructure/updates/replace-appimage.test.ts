// @vitest-environment node
import {
  chmod,
  mkdtemp,
  readFile,
  readdir,
  rm,
  stat,
  writeFile,
} from "node:fs/promises";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { expect, it } from "vitest";
import { replaceAppImage } from "./replace-appimage";

it.skipIf(process.platform !== "linux")(
  "replaces the existing AppImage at its pinned path and preserves its executable permissions",
  async () => {
    const directory = await mkdtemp(join(tmpdir(), "muzo-replace-update-"));
    try {
      const current = join(directory, "Muzo.AppImage");
      const downloaded = join(directory, "downloaded.AppImage");
      await writeFile(current, "old version");
      await chmod(current, 0o744);
      await writeFile(downloaded, "verified new version");
      expect(await replaceAppImage(current, downloaded)).toEqual({
        ok: true,
        value: undefined,
      });
      expect(await readFile(current, "utf8")).toBe("verified new version");
      expect((await stat(current)).mode % 0o1000).toBe(0o744);
      expect(await readFile(downloaded, "utf8")).toBe("verified new version");
      expect((await readdir(directory)).sort()).toEqual([
        "Muzo.AppImage",
        "downloaded.AppImage",
      ]);
      const failed = await replaceAppImage(
        current,
        join(directory, "missing.AppImage"),
      );
      expect(failed).toMatchObject({ ok: false, error: { kind: "storage" } });
      expect(await readFile(current, "utf8")).toBe("verified new version");
    } finally {
      await rm(directory, { recursive: true, force: true });
    }
  },
);
