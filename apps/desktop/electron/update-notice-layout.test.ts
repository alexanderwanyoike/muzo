// @vitest-environment node
import { execFile } from "node:child_process";
import { mkdtemp, readFile, rm, writeFile } from "node:fs/promises";
import { createRequire } from "node:module";
import { tmpdir } from "node:os";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { promisify } from "node:util";
import { build } from "esbuild";
import { expect, it } from "vitest";

it.skipIf(process.platform !== "linux")(
  "downloads through Electron, rejects corrupt bytes and installs at the pinned path only after Restart",
  async () => {
    const directory = await mkdtemp(join(tmpdir(), "muzo-native-update-"));
    try {
      const electronDir = dirname(fileURLToPath(import.meta.url));
      const fixtures = join(electronDir, "fixtures");
      await build({
        entryPoints: [join(fixtures, "update-notice-renderer.tsx")],
        outfile: join(directory, "renderer.js"),
        bundle: true,
        platform: "browser",
        loader: { ".woff2": "file", ".woff": "file" },
        define: { "process.env.NODE_ENV": '"production"' },
      });
      await build({
        entryPoints: [join(fixtures, "verify-update-flow.ts")],
        outfile: join(directory, "runner.cjs"),
        bundle: true,
        platform: "node",
        format: "cjs",
        external: ["electron"],
      });
      await writeFile(
        join(directory, "package.json"),
        JSON.stringify({
          name: "muzo-update-fixture",
          version: "0.0.2",
          main: "runner.cjs",
        }),
      );
      await writeFile(
        join(directory, "index.html"),
        '<!doctype html><html><head><meta charset="UTF-8"><link rel="stylesheet" href="renderer.css"></head><body><div id="root"></div><script src="renderer.js"></script></body></html>',
      );
      const electron: unknown = createRequire(import.meta.url)("electron");
      if (typeof electron !== "string") throw new Error("Electron unavailable");
      await promisify(execFile)(
        "xvfb-run",
        [
          "-a",
          electron,
          "--no-sandbox",
          `--user-data-dir=${join(directory, "profile")}`,
          directory,
          join(electronDir, "preload.cjs"),
        ],
        {
          timeout: 30000,
          maxBuffer: 1024 * 1024,
          env: {
            ...process.env,
            ELECTRON_RUN_AS_NODE: undefined,
            NODE_OPTIONS: undefined,
            XDG_CACHE_HOME: join(directory, "cache"),
          },
        },
      );
      const result = JSON.parse(
        await readFile(join(directory, "result.json"), "utf8"),
      ) as {
        failure?: string;
        corruptionRejected: boolean;
        installed: string;
        layouts: {
          mode: string;
          width: number;
          right: number;
          sidebarRight: number;
          bottom: number;
          height: number;
        }[];
      };
      expect(result.failure).toBeUndefined();
      expect(result.corruptionRejected).toBe(true);
      expect(result.installed).toBe("0.0.3");
      expect(result.layouts).toHaveLength(4);
      for (const layout of result.layouts) {
        expect(layout.right).toBeLessThanOrEqual(layout.sidebarRight);
        expect(layout.bottom).toBeLessThanOrEqual(layout.height);
        expect(layout.bottom).toBeGreaterThan(layout.height - 100);
      }
    } finally {
      await rm(directory, { recursive: true, force: true });
    }
  },
  40000,
);
