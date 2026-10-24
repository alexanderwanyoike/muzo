import { join } from "node:path";

interface DesktopIconApp {
  readonly isPackaged: boolean;
}

export function configureDesktopIcon(
  _app: DesktopIconApp,
  electronDir: string,
  platform: NodeJS.Platform,
): string {
  return join(electronDir, "..", "assets", "icons", platform === "win32" ? "muzo.ico" : "muzo.png");
}
