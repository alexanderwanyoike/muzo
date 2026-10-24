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

const WINDOWS_ACRYLIC_MINIMUM_BUILD = 22621;

export function setDesktopGlass(
  window: GlassWindow,
  enabled: boolean,
  environment: DesktopGlassEnvironment,
): Result<boolean> {
  const windowsAcrylic =
    Number(environment.release.split(".")[2]) >= WINDOWS_ACRYLIC_MINIMUM_BUILD;
  if (enabled && environment.platform === "win32" && !windowsAcrylic)
    return failure(
      "remote",
      "Desktop glass requires Windows 11 22H2 or later. The opaque material is still available.",
    );
  try {
    if (environment.platform === "darwin")
      window.setVibrancy(enabled ? "under-window" : null);
    if (environment.platform === "win32" && windowsAcrylic)
      window.setBackgroundMaterial(enabled ? "acrylic" : "none");
    window.setBackgroundColor(enabled ? "#00000000" : "#ffffff");
    return success(enabled);
  } catch {
    return failure("remote", "Desktop transparency could not be updated.");
  }
}
