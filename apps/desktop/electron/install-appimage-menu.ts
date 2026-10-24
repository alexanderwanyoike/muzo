import { isAbsolute, join } from "node:path";
import { success, type Result } from "./application/result";
import { registerAppImageMenuEntry } from "./appimage-menu-entry";

interface AppImageMenuRuntime {
  platform: NodeJS.Platform;
  isPackaged: boolean;
  homeDirectory: string;
  iconPath: string;
  environment: Partial<Pick<NodeJS.ProcessEnv, "APPIMAGE" | "XDG_DATA_HOME">>;
}

export async function installAppImageMenu(
  runtime: AppImageMenuRuntime,
): Promise<Result<"registered" | "existing" | "skipped">> {
  const appImagePath = runtime.environment.APPIMAGE;
  if (runtime.platform !== "linux" || !runtime.isPackaged || !appImagePath)
    return success("skipped");
  const configured = runtime.environment.XDG_DATA_HOME;
  const dataDirectory = configured && isAbsolute(configured)
    ? configured
    : join(runtime.homeDirectory, ".local/share");
  return registerAppImageMenuEntry({
    appImagePath,
    dataDirectory,
    iconPath: runtime.iconPath,
  });
}
