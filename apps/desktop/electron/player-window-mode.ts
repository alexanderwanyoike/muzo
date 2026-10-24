import type { BrowserWindow } from "electron";
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
>;
interface WindowSnapshot {
  bounds: ReturnType<PlayerWindow["getNormalBounds"]>;
  minimum: number[];
  maximized: boolean;
}

export function createPlayerWindowModeController(window: PlayerWindow) {
  let mode: PlayerWindowMode = "library";
  let library: WindowSnapshot | null = null;
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
      library = next === "library" ? null : savedLibrary;
      mode = next;
      return success(undefined);
    } catch {
      try {
        if (before && !window.isDestroyed()) {
          restore(before);
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
