import { BrowserWindow } from "electron";

export function createPlayerWindow(
  icon: string,
  preload: string,
  platform: NodeJS.Platform = process.platform,
) {
  return new BrowserWindow({
    title: "Muzo",
    transparent: platform === "linux",
    titleBarStyle: platform === "win32" ? "hidden" : "default",
    titleBarOverlay: platform === "win32",
    backgroundColor: "#00000000",
    icon,
    width: 1200,
    height: 800,
    minWidth: 800,
    minHeight: 600,
    webPreferences: {
      contextIsolation: true,
      nodeIntegration: false,
      preload,
    },
  });
}
