import type { BrowserWindow } from "electron";
import { failure, success, type Result } from "./application/result";

type GlassWindow = Pick<
  BrowserWindow,
  | "isDestroyed"
  | "setBackgroundColor"
  | "setVibrancy"
  | "setBackgroundMaterial"
>;
export interface DesktopGlassEnvironment {
  platform: NodeJS.Platform;
  release: string;
  reducedTransparency: boolean;
}

export function setDesktopGlass(
  window: GlassWindow,
  enabled: boolean,
  environment: DesktopGlassEnvironment,
): Result<boolean> {
  try {
    if (environment.platform === "darwin")
      window.setVibrancy(enabled ? "under-window" : null);
    if (environment.platform === "win32")
      window.setBackgroundMaterial(enabled ? "acrylic" : "none");
    window.setBackgroundColor(enabled ? "#00000000" : "#ffffff");
    return success(enabled);
  } catch {
    return failure("remote", "Desktop transparency could not be updated.");
  }
}
