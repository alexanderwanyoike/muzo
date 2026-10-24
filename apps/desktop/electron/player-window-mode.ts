import type { BrowserWindow } from "electron";
import { roundedWindowShape } from "./rounded-window-shape";
import { ConsoleLogger } from "./infrastructure/logging/console-logger";
import type { Logger } from "./application/interfaces/logger-interfaces";
import { failure, success, type Result } from "./application/result";
import type { PlayerWindowMode } from "../shared/player-window-mode";

const COMPACT_DIMENSIONS = {
  compact: { minimumWidth: 360, width: 420 },
  "compact-queue": { minimumWidth: 640, width: 740 },
} as const;
const COMPACT_MINIMUM_HEIGHT = 480;
const COMPACT_HEIGHT = 560;

type PlayerWindow = Pick<
  BrowserWindow,
  | "isDestroyed"
  | "getNormalBounds"
  | "getMinimumSize"
  | "isMaximized"
  | "unmaximize"
  | "maximize"
  | "setMinimumSize"
  | "setContentSize"
  | "setBounds"
  | "getSize"
  | "setShape"
> & {
  on(event: "resize" | "closed", listener: () => void): unknown;
  removeListener(event: "resize" | "closed", listener: () => void): unknown;
};
interface WindowSnapshot {
  bounds: ReturnType<PlayerWindow["getNormalBounds"]>;
  minimum: number[];
  maximized: boolean;
}

export function createPlayerWindowModeController(
  window: PlayerWindow,
  platform: NodeJS.Platform = process.platform,
  logger: Logger = new ConsoleLogger(),
) {
  let mode: PlayerWindowMode = "library";
  let library: WindowSnapshot | null = null;
  const supportsShape =
    platform === "win32" ||
    (platform === "linux" && process.env.XDG_SESSION_TYPE !== "wayland");
  function shape(rounded: boolean): Result<void> {
    if (!supportsShape || window.isDestroyed()) return success(undefined);
    try {
      const [width, height] = window.getSize();
      window.setShape(rounded ? roundedWindowShape(width, height) : []);
      return success(undefined);
    } catch {
      return failure(
        "remote",
        "The mini player window edges could not be updated. Try again.",
      );
    }
  }
  function resize() {
    if (mode === "library") return;
    const updated = shape(true);
    if (!updated.ok) {
      const cleared = shape(false);
      if (!cleared.ok) logger.error(cleared.error.message);
    }
  }
  window.on("resize", resize);
  window.on("closed", () => window.removeListener("resize", resize));
  const snapshot = (): WindowSnapshot => ({
    bounds: window.getNormalBounds(),
    minimum: window.getMinimumSize(),
    maximized: window.isMaximized(),
  });
  function restore(saved: WindowSnapshot) {
    if (window.isMaximized()) window.unmaximize();
    window.setMinimumSize(saved.minimum[0], saved.minimum[1]);
    window.setBounds(saved.bounds);
    if (saved.maximized) window.maximize();
  }
  function setMode(next: unknown): Result<void> {
    if (next !== "library" && next !== "compact" && next !== "compact-queue")
      return failure("invalidInput", "Unknown player window mode.");
    if (window.isDestroyed())
      return failure("remote", "The player window is closed.");
    if (next === mode) return success(undefined);
    let before: WindowSnapshot | null = null;
    try {
      before = snapshot();
      const savedLibrary = mode === "library" ? before : library;
      if (next === "library") {
        if (savedLibrary) restore(savedLibrary);
      } else {
        if (window.isMaximized()) window.unmaximize();
        const dimensions = COMPACT_DIMENSIONS[next];
        window.setMinimumSize(dimensions.minimumWidth, COMPACT_MINIMUM_HEIGHT);
        window.setContentSize(dimensions.width, COMPACT_HEIGHT);
      }
      const shaped = shape(next !== "library");
      if (!shaped.ok) {
        if (before) restore(before);
        const rolledBack = shape(mode !== "library");
        return rolledBack.ok ? shaped : rolledBack;
      }
      library = next === "library" ? null : savedLibrary;
      mode = next;
      return success(undefined);
    } catch {
      try {
        if (before && !window.isDestroyed()) {
          restore(before);
          const rolledBack = shape(mode !== "library");
          if (!rolledBack.ok) return rolledBack;
        }
      } catch {
        return failure(
          "remote",
          "The player window could not be restored. Try again or restart Muzo.",
        );
      }
      return failure(
        "remote",
        "The player window could not be resized. Try again.",
      );
    }
  }
  return { setMode };
}
