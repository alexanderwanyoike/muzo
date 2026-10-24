import { BrowserWindow } from "electron";

export function createPlayerWindow(icon: string, preload: string) {
  return new BrowserWindow({
    title: "Muzo",
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
