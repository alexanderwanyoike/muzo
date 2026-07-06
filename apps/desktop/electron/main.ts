import { app, BrowserWindow, dialog, ipcMain } from "electron";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

import { handleElectronCommand } from "./commands";

const electronDir = dirname(fileURLToPath(import.meta.url));
const devUrl = process.env.MUZO_ELECTRON_DEV_URL ?? "http://localhost:1420";

async function createWindow() {
  const window = new BrowserWindow({
    title: "Muzo",
    width: 1200,
    height: 800,
    minWidth: 800,
    minHeight: 600,
    webPreferences: {
      contextIsolation: true,
      nodeIntegration: false,
      preload: join(electronDir, "preload.cjs"),
    },
  });

  if (app.isPackaged) {
    await window.loadFile(join(electronDir, "..", "dist", "index.html"));
  } else {
    await window.loadURL(devUrl);
  }
}

ipcMain.handle("muzo:invoke", (_event, command: string, args?: unknown) =>
  handleElectronCommand(command, args),
);

ipcMain.handle("muzo:open-directory", async () => {
  const result = await dialog.showOpenDialog({
    properties: ["openDirectory"],
  });
  return result.canceled ? null : result.filePaths[0] ?? null;
});

app.whenReady().then(() => {
  void createWindow();

  app.on("activate", () => {
    if (BrowserWindow.getAllWindows().length === 0) {
      void createWindow();
    }
  });
});

app.on("window-all-closed", () => {
  if (process.platform !== "darwin") {
    app.quit();
  }
});
