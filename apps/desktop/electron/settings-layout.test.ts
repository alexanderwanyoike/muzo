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

interface Measurement {
  width: number;
  connected: boolean;
  theme: string;
  mode: string;
  layout: { sections: { name: string; left: number; width: number; top: number; bottom: number; headingLeft: number }[]; controls: number[]; overflow: number };
}

it("aligns Settings sections and controls without overflow at desktop and minimum sizes", async () => {
  const directory = mkdtempSync(join(tmpdir(), "muzo-settings-layout-"));
  try {
    const fixtures = join(dirname(fileURLToPath(import.meta.url)), "fixtures");
    const runner = join(directory, "runner.cjs");
    const page = join(directory, "index.html");
    const resultPath = join(directory, "result.json");
    await build({ entryPoints: [join(fixtures, "settings-layout-renderer.tsx")], outfile: join(directory, "renderer.js"), bundle: true, platform: "browser", loader: { ".woff2": "file", ".woff": "file" }, define: { "process.env.NODE_ENV": '"production"' } });
    await build({ entryPoints: [join(fixtures, "verify-settings-layout.ts")], outfile: runner, bundle: true, platform: "node", format: "cjs", external: ["electron"] });
    writeFileSync(page, '<!doctype html><html><head><meta charset="UTF-8"><link rel="stylesheet" href="renderer.css"></head><body><div id="root"></div><script src="renderer.js"></script></body></html>');
    const electron: unknown = createRequire(import.meta.url)("electron");
    if (typeof electron !== "string") throw new Error("Electron executable unavailable");
    const args = ["--no-sandbox", `--user-data-dir=${join(directory, "profile")}`, runner, page, resultPath];
    await promisify(execFile)(process.platform === "linux" ? "xvfb-run" : electron, process.platform === "linux" ? ["-a", electron, ...args] : args, { timeout: 30_000, env: { ...process.env, ELECTRON_RUN_AS_NODE: undefined, NODE_OPTIONS: undefined } });
    const measurements = JSON.parse(readFileSync(resultPath, "utf8")) as Measurement[];
    expect(measurements).toHaveLength(24);
    for (const { width, connected, theme, mode, layout } of measurements) {
      const context = `${width}px, ${theme}/${mode}, Dropbox ${connected ? "connected" : "disconnected"}`;
      expect(layout.sections.map(({ name }) => name), context).toEqual(["Appearance", "Dropbox", "Library folders", "Indexed sources"]);
      for (const section of layout.sections) {
        expect(section.width, `${context}, ${section.name} width`).toBeCloseTo(layout.sections[0].width, 0);
        expect(section.left, `${context}, ${section.name} edge`).toBeCloseTo(layout.sections[0].left, 0);
        expect(section.headingLeft, `${context}, ${section.name} heading`).toBeCloseTo(layout.sections[0].headingLeft, 0);
      }
      for (let index = 1; index < layout.sections.length; index += 1) {
        expect(layout.sections[index].top - layout.sections[index - 1].bottom, `${context}, section spacing`).toBeCloseTo(20, 0);
      }
      expect(layout.overflow, context).toBeLessThanOrEqual(1);
      expect(Math.max(...layout.controls) - Math.min(...layout.controls), `${context}, control heights`).toBeLessThanOrEqual(1);
    }
  } finally { rmSync(directory, { recursive: true, force: true }); }
}, 40_000);
