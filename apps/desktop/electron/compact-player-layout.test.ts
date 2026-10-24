// @vitest-environment node
import { execFile } from "node:child_process";
import { mkdtempSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { createRequire } from "node:module";
import { tmpdir } from "node:os";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { promisify } from "node:util";
import { build } from "esbuild";
import { expect, it } from "vitest";

it("fits compact controls at native minimum sizes and restores the library window through the real preload", async () => {
  const directory = mkdtempSync(join(tmpdir(), "muzo-compact-layout-"));
  try {
    const electronDir = dirname(fileURLToPath(import.meta.url));
    const fixtures = join(electronDir, "fixtures");
    await build({
      entryPoints: [join(fixtures, "compact-player-renderer.tsx")],
      outfile: join(directory, "renderer.js"),
      bundle: true,
      platform: "browser",
      loader: { ".woff2": "file", ".woff": "file" },
      define: { "process.env.NODE_ENV": '"production"' },
    });
    const runner = join(directory, "runner.cjs");
    await build({
      entryPoints: [join(fixtures, "verify-compact-player.ts")],
      outfile: runner,
      bundle: true,
      platform: "node",
      format: "cjs",
      external: ["electron"],
    });
    const page = join(directory, "index.html");
    writeFileSync(
      page,
      '<!doctype html><html><head><meta charset="UTF-8"><link rel="stylesheet" href="renderer.css"></head><body><div id="root"></div><script src="renderer.js"></script></body></html>',
    );
    const resultPath = join(directory, "result.json");
    const electron: unknown = createRequire(import.meta.url)("electron");
    if (typeof electron !== "string")
      throw new Error("Electron executable unavailable");
    const args = [
      "--no-sandbox",
      `--user-data-dir=${join(directory, "profile")}`,
      runner,
      page,
      join(electronDir, "preload.cjs"),
      resultPath,
    ];
    await promisify(execFile)(
      process.platform === "linux" ? "xvfb-run" : electron,
      process.platform === "linux" ? ["-a", electron, ...args] : args,
      {
        timeout: 20000,
        env: {
          ...process.env,
          ELECTRON_RUN_AS_NODE: undefined,
          NODE_OPTIONS: undefined,
        },
      },
    );
    const result = JSON.parse(readFileSync(resultPath, "utf8")) as {
      measurements: {
        mode: string;
        queue: boolean;
        layout: {
          width: number;
          height: number;
          overflow: number;
          libraryHidden: boolean;
          controls: {
            name: string;
            left: number;
            right: number;
            top: number;
            bottom: number;
          }[];
        };
      }[];
      original: unknown;
      restored: unknown;
      minimum: number[];
    };
    expect(result.measurements).toHaveLength(8);
    expect(result.restored).toEqual(result.original);
    expect(result.minimum).toEqual([800, 600]);
    for (const { mode, queue, layout } of result.measurements) {
      const context = `${mode} ${layout.width}x${layout.height} ${queue ? "queue" : "compact"}`;
      expect(layout.overflow, context).toBeLessThanOrEqual(0);
      expect(layout.libraryHidden, context).toBe(true);
      expect(layout.controls.length, context).toBeGreaterThanOrEqual(7);
      for (const control of layout.controls) {
        expect(
          control.left,
          `${context} ${control.name}`,
        ).toBeGreaterThanOrEqual(0);
        expect(control.right, `${context} ${control.name}`).toBeLessThanOrEqual(
          layout.width,
        );
        expect(
          control.top,
          `${context} ${control.name}`,
        ).toBeGreaterThanOrEqual(0);
        expect(
          control.bottom,
          `${context} ${control.name}`,
        ).toBeLessThanOrEqual(layout.height);
      }
    }
  } finally {
    rmSync(directory, { recursive: true, force: true });
  }
}, 25000);
