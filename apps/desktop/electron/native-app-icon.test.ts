// @vitest-environment node
import { execFile } from "node:child_process";
import { mkdtempSync, readFileSync, rmSync } from "node:fs";
import { createRequire } from "node:module";
import { tmpdir } from "node:os";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { promisify } from "node:util";
import { build } from "esbuild";
import { expect, it } from "vitest";

it("decodes the platform icon at taskbar sizes and creates a native window", async () => {
  const directory = mkdtempSync(join(tmpdir(), "muzo-native-icon-"));
  try {
    const electronDir = dirname(fileURLToPath(import.meta.url));
    const bundle = join(directory, "icon.cjs");
    const resultPath = join(directory, "result.json");
    await build({
      entryPoints: [join(electronDir, "fixtures", "verify-native-icon.ts")],
      outfile: bundle,
      bundle: true,
      platform: "node",
      format: "cjs",
      external: ["electron"],
    });
    const electron: unknown = createRequire(import.meta.url)("electron");
    if (typeof electron !== "string")
      throw new Error("Electron executable unavailable");
    const args = [
      "--no-sandbox",
      `--user-data-dir=${join(directory, "profile")}`,
      bundle,
      electronDir,
      resultPath,
    ];
    const executable = process.platform === "linux" ? "xvfb-run" : electron;
    await promisify(execFile)(
      executable,
      process.platform === "linux" ? ["-a", electron, ...args] : args,
      {
        timeout: 30_000,
        env: {
          ...process.env,
          ELECTRON_RUN_AS_NODE: undefined,
          NODE_OPTIONS: undefined,
        },
      },
    );
    expect(JSON.parse(readFileSync(resultPath, "utf8"))).toEqual({
      identity: { name: "Muzo", profile: "Muzo", session: "Muzo" },
      decoded: true,
      sizes: [16, 32, 256].map((size) => ({ width: size, height: size })),
      windowCreated: true,
    });
  } finally {
    rmSync(directory, { recursive: true, force: true });
  }
}, 40_000);
