import type { BrowserWindow } from "electron";
import { failure, success, type Result } from "./application/result";
import type { PlayerWindowMode } from "../shared/player-window-mode";

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

export function createPlayerWindowModeController(window: PlayerWindow) {
  let mode: PlayerWindowMode = "library";
  let library: {
    bounds: ReturnType<PlayerWindow["getNormalBounds"]>;
    minimum: number[];
    maximized: boolean;
  } | null = null;
  function setMode(next: unknown): Result<void> {
    if (next !== "library" && next !== "compact" && next !== "compact-queue")
      return failure("invalidInput", "Unknown player window mode.");
    if (window.isDestroyed())
      return failure("remote", "The player window is closed.");
    if (next === mode) return success(undefined);
    try {
      if (next === "library") {
        if (library) {
          window.setMinimumSize(library.minimum[0], library.minimum[1]);
          window.setBounds(library.bounds);
          if (library.maximized) window.maximize();
        }
        library = null;
      } else {
        if (mode === "library") {
          library = {
            bounds: window.getNormalBounds(),
            minimum: window.getMinimumSize(),
            maximized: window.isMaximized(),
          };
          if (library.maximized) window.unmaximize();
        }
        window.setMinimumSize(next === "compact-queue" ? 640 : 360, 480);
        window.setContentSize(next === "compact-queue" ? 740 : 420, 560);
      }
      mode = next;
      return success(undefined);
    } catch {
      return failure("remote", "The player window could not be resized.");
    }
  }
  return { setMode };
}
