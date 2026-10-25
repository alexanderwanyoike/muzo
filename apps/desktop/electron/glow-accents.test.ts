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
import { MATERIALS, MODES } from "../src/appearance";
import { contrastRatio } from "./fixtures/contrast-ratio";

it("renders readable glowing playback controls in both player views, clears classic effects and respects forced colours", async () => {
  const directory = await mkdtemp(join(tmpdir(), "muzo-glow-controls-"));
  try {
    const fixtures = join(dirname(fileURLToPath(import.meta.url)), "fixtures");
    await build({
      entryPoints: [join(fixtures, "accent-controls-renderer.tsx")],
      outfile: join(directory, "renderer.js"),
      bundle: true,
      platform: "browser",
      loader: { ".woff2": "file", ".woff": "file" },
      define: { "process.env.NODE_ENV": '"production"' },
    });
    const runner = join(directory, "runner.cjs");
    await build({
      entryPoints: [join(fixtures, "verify-glow-controls.ts")],
      outfile: runner,
      bundle: true,
      platform: "node",
      format: "cjs",
      external: ["electron"],
    });
    const page = join(directory, "index.html");
    const resultPath = join(directory, "result.json");
    await writeFile(
      page,
      '<!doctype html><html><head><meta charset="UTF-8"><link rel="stylesheet" href="renderer.css"></head><body><div id="root"></div><script src="renderer.js"></script></body></html>',
    );
    const electron: unknown = createRequire(import.meta.url)("electron");
    if (typeof electron !== "string") throw new Error("Electron unavailable");
    const args = [
      "--no-sandbox",
      `--user-data-dir=${join(directory, "profile")}`,
      runner,
      page,
      resultPath,
    ];
    await promisify(execFile)(
      process.platform === "linux" ? "xvfb-run" : electron,
      process.platform === "linux" ? ["-a", electron, ...args] : args,
      {
        timeout: 60000,
        env: {
          ...process.env,
          ELECTRON_RUN_AS_NODE: undefined,
          NODE_OPTIONS: undefined,
        },
      },
    );
    const result = JSON.parse(await readFile(resultPath, "utf8")) as {
      failure?: string;
      measurements: {
        theme: string;
        mode: string;
        compact: boolean;
        accent: string;
        foreground: string;
        background: string;
        shadow: string;
        filter: string;
      }[];
      restored: {
        original: { shadow: string; filter: string };
        after: { shadow: string; filter: string };
      }[];
      forced: { shadow: string; filter: string };
    };
    expect(result.failure).toBeUndefined();
    expect(result.measurements).toHaveLength(
      MATERIALS.length * MODES.length * 2 * 6,
    );
    for (const sample of result.measurements) {
      const context = `${sample.theme}/${sample.mode}/${sample.compact ? "compact" : "full"}/${sample.accent}`;
      expect(
        contrastRatio(sample.foreground, sample.background),
        context,
      ).toBeGreaterThanOrEqual(4.5);
      expect(sample.shadow, context).not.toBe("none");
      expect(sample.filter, context).not.toBe("none");
    }
    expect(result.restored).toHaveLength(MATERIALS.length * MODES.length * 2);
    for (const sample of result.restored)
      expect(sample.after).toEqual(sample.original);
    expect(result.forced).toEqual({ shadow: "none", filter: "none" });
  } finally {
    await rm(directory, { recursive: true, force: true });
  }
}, 65000);
