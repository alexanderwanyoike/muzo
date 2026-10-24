import { BrowserWindow } from "electron";

export function createPlayerWindow(icon: string, preload: string) {
  return new BrowserWindow({
    title: "Muzo",
    transparent: process.platform === "linux",
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
