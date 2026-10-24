import type { Dock } from "electron";
import { join } from "node:path";

interface DesktopIconApp {
  readonly isPackaged: boolean;
  readonly dock?: Pick<Dock, "setIcon">;
}

export function configureDesktopIcon(
  app: DesktopIconApp,
  electronDir: string,
  platform: NodeJS.Platform,
): string {
  const platformIcons: Partial<Record<NodeJS.Platform, string>> = {
    win32: "muzo.ico",
    linux: "muzo-window.png",
  };
  const path = join(
    electronDir,
    "..",
    "assets",
    "icons",
    platformIcons[platform] ?? "muzo.png",
  );
  if (platform === "darwin" && !app.isPackaged) {
    app.dock?.setIcon(path);
  }
  return path;
}
