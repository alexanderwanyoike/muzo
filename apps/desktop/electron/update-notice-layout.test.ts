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
import type { measureUpdateStatusBar } from "./fixtures/measure-update-status-bar";
import { MATERIALS, MODES } from "../src/appearance";

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
        layouts: (ReturnType<typeof measureUpdateStatusBar> & {
          theme: string;
          mode: string;
          requestedWidth: number;
        })[];
      };
      expect(result.failure).toBeUndefined();
      expect(result.corruptionRejected).toBe(true);
      expect(result.installed).toBe("0.0.3");
      expect(result.layouts).toHaveLength(MATERIALS.length * MODES.length * 2);
      for (const layout of result.layouts) {
        expect(layout.width).toBe(layout.requestedWidth);
        expect(layout.barLeft).toBe(0);
        expect(layout.barRight).toBe(layout.width);
        expect(layout.barBottom).toBe(layout.height);
        expect(layout.barHeight).toBeLessThanOrEqual(36);
        expect(layout.sidebarBottom).toBe(layout.barTop);
        expect(layout.contentBottom).toBe(layout.barTop);
        expect(layout.right).toBeGreaterThanOrEqual(layout.width - 16);
        expect(layout.right).toBeLessThanOrEqual(layout.width - 8);
        expect(layout.top).toBeGreaterThanOrEqual(layout.barTop + 3);
        expect(layout.bottom).toBeLessThanOrEqual(layout.height - 3);
        expect(layout.buttonHeight).toBe(24);
        expect(layout.overflow).toBe(false);
        expect(layout.focused).toBe(true);
        expect(layout.outlineWidth).toBe("2px");
        expect(layout.outlineOffset).toBe("1px");
      }
    } finally {
      await rm(directory, { recursive: true, force: true });
    }
  },
  40000,
);
