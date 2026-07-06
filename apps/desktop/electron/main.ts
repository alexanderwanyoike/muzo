import { app, BrowserWindow, dialog, ipcMain } from "electron";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

import { createElectronContainer } from "./composition/electron-container";

const electronDir = dirname(fileURLToPath(import.meta.url));
const devUrl = process.env.MUZO_ELECTRON_DEV_URL ?? "http://localhost:1420";
const container = createElectronContainer();
const commandDispatcher = container.resolve("commandDispatcher");
const migrateDatabase = container.resolve("migrateDatabase");

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

ipcMain.handle("muzo:invoke", async (_event, command: string, args?: unknown) => {
  try {
    return {
      ok: true,
      value: await commandDispatcher.handleElectronCommand(command, args),
    };
  } catch (error) {
    return {
      ok: false,
      error,
    };
  }
});

ipcMain.handle("muzo:open-directory", async () => {
  const result = await dialog.showOpenDialog({
    properties: ["openDirectory"],
  });
  return result.canceled ? null : result.filePaths[0] ?? null;
});

app.whenReady().then(async () => {
  await migrateDatabase();
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
