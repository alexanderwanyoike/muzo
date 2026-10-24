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

interface FocusState {
  focused: boolean;
  focusVisible: boolean;
  outline: string;
  outlineWidth: string;
  value: number;
}

it("keeps mouse seeking free of focus borders while showing keyboard focus and accepting arrow keys", async () => {
  const directory = mkdtempSync(join(tmpdir(), "muzo-seek-focus-"));
  try {
    const fixtures = join(dirname(fileURLToPath(import.meta.url)), "fixtures");
    await build({
      entryPoints: [join(fixtures, "seek-focus-renderer.tsx")],
      outfile: join(directory, "renderer.js"),
      bundle: true,
      platform: "browser",
      define: { "process.env.NODE_ENV": '"production"' },
    });
    const runner = join(directory, "runner.cjs");
    await build({
      entryPoints: [join(fixtures, "verify-seek-focus.ts")],
      outfile: runner,
      bundle: true,
      platform: "node",
      format: "cjs",
      external: ["electron"],
    });
    const page = join(directory, "index.html");
    writeFileSync(
      page,
      '<!doctype html><html><head><link rel="stylesheet" href="renderer.css"></head><body><div id="root"></div><script src="renderer.js"></script></body></html>',
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
      resultPath,
    ];
    await promisify(execFile)(
      process.platform === "linux" ? "xvfb-run" : electron,
      process.platform === "linux" ? ["-a", electron, ...args] : args,
      {
        timeout: 15_000,
        env: {
          ...process.env,
          ELECTRON_RUN_AS_NODE: undefined,
          NODE_OPTIONS: undefined,
        },
      },
    );
    const measurements = JSON.parse(readFileSync(resultPath, "utf8")) as {
      mode: string;
      pointer: FocusState;
      keyboard: FocusState;
      adjusted: FocusState;
    }[];
    expect(measurements).toHaveLength(2);
    for (const { mode, pointer, keyboard, adjusted } of measurements) {
      expect(pointer.focused, mode).toBe(true);
      expect(pointer.focusVisible, mode).toBe(false);
      expect(pointer.outline, `${mode}, mouse seeking`).toBe("none");
      expect(keyboard.focused, mode).toBe(true);
      expect(keyboard.focusVisible, mode).toBe(true);
      expect(keyboard.outline, `${mode}, keyboard focus`).toBe("solid");
      expect(keyboard.outlineWidth, mode).toBe("2px");
      expect(adjusted.value, `${mode}, keyboard seeking`).toBeGreaterThan(
        keyboard.value,
      );
    }
  } finally {
    rmSync(directory, { recursive: true, force: true });
  }
}, 20_000);
