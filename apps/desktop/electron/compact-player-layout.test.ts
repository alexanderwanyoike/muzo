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
    const execution = await promisify(execFile)(
      process.platform === "linux" ? "xvfb-run" : electron,
      process.platform === "linux" ? ["-a", electron, ...args] : args,
      {
        timeout: 40000,
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
    if (execution.stderr.includes("Compact fixture timeout"))
      throw new Error(execution.stderr);
    const result = JSON.parse(readFileSync(resultPath, "utf8")) as {
      failure?: string;
      measurements: {
        nativeShape: string | null;
        theme: string;
        mode: string;
        queue: boolean;
        queueAccess?: {
          scrollTop: number;
          lastBottom: number;
          listBottom: number;
        };
        layout: {
          timelineBackground: string;
          contrastSamples: {
            text: string;
            muted: string;
            backgrounds: string[];
          }[];
          bottomCorners: string[];
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
      interactionStates: {
        mode: string;
        foreground: string;
        selected: string;
        hover: { background: string; color: string };
        focus: { visible: boolean; color: string; style: string };
      }[];
      libraryControls: {
        width: number;
        top: number;
        right: number;
        bottom: number;
        buttonWidth: number;
        volumeTop: number;
        text: string;
        title: string;
      }[];
      fallback: { queueAlpha: number };
      restoredShape: string | null;
      original: unknown;
      restored: unknown;
      minimum: number[];
    };
    expect(result.failure).toBeUndefined();
    expect(result.interactionStates).toHaveLength(4);
    for (const state of result.interactionStates) {
      const dark = state.mode === "dark";
      expect(state.foreground, `${state.mode} inactive navigation`).toBe(
        dark ? "rgb(179, 179, 179)" : "rgb(85, 85, 85)",
      );
      expect(state.selected, `${state.mode} selected navigation`).toBe(
        dark ? "rgb(42, 42, 42)" : "rgb(216, 216, 216)",
      );
      expect(state.hover.background, `${state.mode} neutral hover`).toBe(
        dark ? "rgb(42, 42, 42)" : "rgb(222, 222, 222)",
      );
      expect(state.hover.color, `${state.mode} readable hover`).toBe(
        dark ? "rgb(245, 245, 245)" : "rgb(23, 23, 23)",
      );
      expect(state.focus.visible, `${state.mode} keyboard focus`).toBe(true);
      expect(state.focus.style, `${state.mode} focus ring`).toBe("solid");
      expect(state.focus.color, `${state.mode} neutral focus ring`).toBe(
        dark ? "rgb(255, 255, 255)" : "rgb(23, 23, 23)",
      );
    }
    expect(result.measurements).toHaveLength(24);
    expect(result.fallback.queueAlpha).toBe(255);
    expect(result.libraryControls).toHaveLength(12);
    for (const control of result.libraryControls) {
      expect(control.text).toBe("");
      expect(control.title).toBe("Mini player");
      expect(control.top).toBeGreaterThanOrEqual(0);
      expect(control.top).toBeLessThanOrEqual(8);
      expect(control.buttonWidth).toBeGreaterThanOrEqual(24);
      expect(control.width - control.right).toBeGreaterThanOrEqual(8);
      expect(control.width - control.right).toBeLessThanOrEqual(16);
      expect(control.bottom).toBeLessThan(control.volumeTop);
    }
    expect(result.restored).toEqual(result.original);
    if (process.platform === "linux") {
      expect(result.restoredShape).toContain("No window shape defined");
      for (const measurement of result.measurements) {
        expect(measurement.nativeShape).toContain("No window shape defined");
      }
    }
    expect(result.minimum).toEqual([800, 600]);
    for (const {
      theme,
      mode,
      queue,
      layout,
      queueAccess,
    } of result.measurements) {
      if (theme === "pulse")
        expect(
          layout.timelineBackground,
          `${mode} Pulse timeline stays unboxed`,
        ).toBe("rgba(0, 0, 0, 0)");
      if (queue) {
        expect(queueAccess?.scrollTop).toBeGreaterThan(0);
        expect(queueAccess?.lastBottom).toBeLessThanOrEqual(
          (queueAccess?.listBottom ?? 0) + 1,
        );
      }
      if (theme === "glass") {
        for (const sample of layout.contrastSamples) {
          for (const background of sample.backgrounds) {
            expect(
              contrastRatio(sample.text, background),
              `${mode} glass text`,
            ).toBeGreaterThanOrEqual(4.5);
            expect(
              contrastRatio(sample.muted, background),
              `${mode} glass muted`,
            ).toBeGreaterThanOrEqual(4.5);
          }
        }
      }
      const context = `${mode} ${layout.width}x${layout.height} ${queue ? "queue" : "compact"}`;
      expect(layout.bottomCorners, context).toEqual(["0px", "0px"]);
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
}, 45000);
