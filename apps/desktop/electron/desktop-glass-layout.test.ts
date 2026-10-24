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
import { contrastRatio } from "./fixtures/contrast-ratio";

it("reveals a changing desktop through the native library and mini player while preserving opaque themes", async () => {
  const directory = mkdtempSync(join(tmpdir(), "muzo-desktop-glass-"));
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
      entryPoints: [join(fixtures, "verify-desktop-glass.ts")],
      outfile: runner,
      bundle: true,
      platform: "node",
      format: "cjs",
      external: ["electron"],
    });
    const page = join(directory, "index.html");
    const resultPath = join(directory, "result.json");
    writeFileSync(
      page,
      '<!doctype html><html><head><meta charset="UTF-8"><link rel="stylesheet" href="renderer.css"></head><body><div id="root"></div><script src="renderer.js"></script></body></html>',
    );
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
      join(electronDir, "../assets/icons/muzo.png"),
    ];
    await promisify(execFile)(
      process.platform === "linux" ? "xvfb-run" : electron,
      process.platform === "linux"
        ? [
            "-a",
            "-s",
            "-screen 0 1600x1100x24 +extension Composite",
            electron,
            ...args,
          ]
        : args,
      {
        timeout: 40_000,
        env: {
          ...process.env,
          ELECTRON_RUN_AS_NODE: undefined,
          NODE_OPTIONS: undefined,
          XDG_SESSION_TYPE:
            process.platform === "linux" ? "x11" : process.env.XDG_SESSION_TYPE,
          WAYLAND_DISPLAY: undefined,
        },
      },
    );
    const result = JSON.parse(readFileSync(resultPath, "utf8")) as {
      failure?: string;
      samples: {
        mode: string;
        view: string;
        alpha: number;
        colour: number[];
        text: string;
        muted: string;
      }[];
      restoredAlpha: number;
      restoredSize: number[];
      originalSize: number[];
      backdropDifference: number;
      desktopCompositingError: number;
      reducedAlpha: number;
      overlayChecks: { visible: boolean; collisions: string[] }[];
    };
    expect(result.failure).toBeUndefined();
    expect(result.samples).toHaveLength(6);
    for (const sample of result.samples) {
      const context = `${sample.mode}/${sample.view}`;
      expect(sample.alpha, context).toBeGreaterThan(0);
      expect(sample.alpha, context).toBeLessThan(255);
      for (const backdrop of [0, 255]) {
        const colour = `#${sample.colour
          .map((channel) =>
            Math.round(
              (channel * sample.alpha) / 255 +
                backdrop * (1 - sample.alpha / 255),
            )
              .toString(16)
              .padStart(2, "0"),
          )
          .join("")}`;
        expect(
          contrastRatio(sample.text, colour),
          `${context}, text over ${backdrop}`,
        ).toBeGreaterThanOrEqual(4.5);
        expect(
          contrastRatio(sample.muted, colour),
          `${context}, muted over ${backdrop}`,
        ).toBeGreaterThanOrEqual(4.5);
      }
    }
    expect(result.restoredAlpha).toBe(255);
    expect(result.reducedAlpha).toBe(255);
    expect(result.restoredSize).toEqual(result.originalSize);
    expect(result.overlayChecks).toHaveLength(
      process.platform === "darwin" ? 0 : 2,
    );
    for (const overlay of result.overlayChecks) {
      expect(overlay.visible).toBe(true);
      expect(overlay.collisions).toEqual([]);
    }
    if (process.platform === "linux") {
      expect(result.backdropDifference).toBeGreaterThan(10);
      expect(result.desktopCompositingError).toBeLessThanOrEqual(6);
    }
  } finally {
    rmSync(directory, { recursive: true, force: true });
  }
}, 45_000);
