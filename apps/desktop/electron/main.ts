import {
  app,
  BrowserWindow,
  dialog,
  ipcMain,
  Menu,
  nativeImage,
  nativeTheme,
  safeStorage,
  shell,
} from "electron";
import { release } from "node:os";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

import { createElectronContainer } from "./composition/electron-container";
import { ElectronSafeStorageDropboxTokenCipher } from "./infrastructure/dropbox/electron-safe-storage-dropbox-token-cipher";
import { ElectronArtworkThumbnails } from "./infrastructure/metadata/electron-artwork-thumbnails";
import { configureDesktopIdentity } from "./desktop-identity";
import { configureDesktopIcon } from "./desktop-icon";
import { createPlayerWindow } from "./create-player-window";
import { installDesktopGlass } from "./install-desktop-glass";
import { installPlayerWindowMode } from "./install-player-window-mode";
import { installDesktopMenu } from "./desktop-menu";
import { runStartupTasks } from "./startup";

configureDesktopIdentity(app);

const electronDir = dirname(fileURLToPath(import.meta.url));
const devUrl = process.env.MUZO_ELECTRON_DEV_URL ?? "http://localhost:1420";
const container = createElectronContainer({
  artworkThumbnails: new ElectronArtworkThumbnails(nativeImage),
  dropboxCipher: new ElectronSafeStorageDropboxTokenCipher(safeStorage),
  openExternal: (url) => shell.openExternal(url),
});
const commandDispatcher = container.resolve("commandDispatcher");
const migrateDatabase = container.resolve("migrateDatabase");
const reconcileFilesystemLibraries = container.resolve(
  "reconcileFilesystemLibraries",
);
const watchFilesystemLibraries = container.resolve("watchFilesystemLibraries");
const logger = container.resolve("logger");

async function createWindow(icon: string) {
  const window = createPlayerWindow(icon, join(electronDir, "preload.cjs"));

  if (app.isPackaged) {
    await window.loadFile(join(electronDir, "..", "dist", "index.html"));
  } else {
    await window.loadURL(devUrl);
  }
}

installPlayerWindowMode(ipcMain, (event) =>
  BrowserWindow.fromWebContents(event.sender),
);

installDesktopGlass(
  ipcMain,
  (event) => BrowserWindow.fromWebContents(event.sender),
  () => ({
    platform: process.platform,
    release: release(),
    reducedTransparency: nativeTheme.prefersReducedTransparency,
  }),
);

ipcMain.handle(
  "muzo:invoke",
  async (_event, command: string, args?: unknown) => {
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
  },
);

ipcMain.handle("muzo:open-directory", async () => {
  const result = await dialog.showOpenDialog({
    properties: ["openDirectory"],
  });
  return result.canceled ? null : (result.filePaths[0] ?? null);
});

app.whenReady().then(async () => {
  installDesktopMenu(Menu, process.platform);
  const icon = configureDesktopIcon(app, electronDir, process.platform);
  await runStartupTasks({
    migrateDatabase,
    reconcileFilesystemLibraries,
    watchFilesystemLibraries,
    logger,
  });
  container.resolve("dropboxPoller").start();
  void createWindow(icon);

  app.on("activate", () => {
    if (BrowserWindow.getAllWindows().length === 0) {
      void createWindow(icon);
    }
  });
});

app.on("before-quit", () => container.resolve("dropboxPoller").stop());

app.on("window-all-closed", () => {
  if (process.platform !== "darwin") {
    app.quit();
  }
});
