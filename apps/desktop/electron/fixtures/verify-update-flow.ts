import { app, BrowserWindow, ipcMain } from "electron";
import { execFile } from "node:child_process";
import { createHash } from "node:crypto";
import { chmod, copyFile, mkdir, readFile, writeFile } from "node:fs/promises";
import { createServer } from "node:http";
import { join } from "node:path";
import { promisify } from "node:util";
import electronUpdater from "electron-updater";
import { MATERIALS, MODES } from "../../src/appearance";
import { measureUpdateStatusBar } from "./measure-update-status-bar";
import { createApplicationUpdates } from "../application/application-updates";
import { success } from "../application/result";
import { createNativeUpdateBackend } from "../infrastructure/updates/native-update-backend";
import { replaceAppImage } from "../infrastructure/updates/replace-appimage";
import { installApplicationUpdates } from "../install-application-updates";
import { prepareUpdaterConfig } from "../distribution/prepare-updater-config";

const directory = app.getAppPath();
const payload = Buffer.from('#!/bin/sh\nprintf 0.0.3 > "$1"\n');
const sha512 = createHash("sha512").update(payload).digest("base64");
let corrupt = true;
const server = createServer((request, response) => {
  if (request.url?.startsWith("/latest-linux.yml")) {
    response.end(
      `version: 0.0.3\nfiles:\n  - url: Muzo-0.0.3-linux-x64.AppImage\n    sha512: ${sha512}\n    size: ${payload.length}\npath: Muzo-0.0.3-linux-x64.AppImage\nsha512: ${sha512}\n`,
    );
  } else {
    const bytes = corrupt ? Buffer.alloc(payload.length) : payload;
    response.writeHead(200, { "Content-Length": bytes.length });
    response.end(bytes);
  }
});
ipcMain.handle("muzo:invoke", () => ({ ok: true, value: [] }));

async function waitFor(window: BrowserWindow, state: string) {
  await window.webContents.executeJavaScript(`new Promise((resolve, reject) => {
 const deadline = Date.now() + 8000;
 function check() { const button = document.querySelector('.update-notice button'); if (button?.getAttribute('aria-label')?.startsWith(${JSON.stringify(state)})) resolve(); else if (Date.now() > deadline) reject(new Error('Update fixture timeout: ' + ${JSON.stringify(state)})); else setTimeout(check, 25); }
 check(); })`);
}
async function click(window: BrowserWindow) {
  await window.webContents.executeJavaScript(
    "document.querySelector('.update-notice button').click()",
  );
}
void app.whenReady().then(async () => {
  try {
    await new Promise<void>((resolve) => {
      server.listen(0, "127.0.0.1", resolve);
    });
    const address = server.address();
    if (!address || typeof address === "string")
      throw new Error("Update server missing");
    const current = join(directory, "Pinned Muzo.AppImage");
    const marker = join(directory, "installed.txt");
    await writeFile(current, "original executable");
    await chmod(current, 0o744);
    process.env.APPIMAGE = current;
    const configured = await prepareUpdaterConfig(directory);
    if (!configured.ok) throw new Error(configured.error.message);
    await copyFile(
      join(directory, "resources", "app-update.yml"),
      join(directory, "dev-app-update.yml"),
    );
    const client = new electronUpdater.AppImageUpdater({
      provider: "generic",
      url: `http://127.0.0.1:${address.port}`,
    });
    client.forceDevUpdateConfig = true;
    client.logger = null;
    let confirmed = false;
    const backend = createNativeUpdateBackend({
      enabled: true,
      automatic: true,
      createClient: () => client,
      checkRelease: async () => success({ version: "0.0.3" }),
      confirmRestart: async () => confirmed,
      openRelease: async () => {},
      async installDownloaded(path) {
        const result = await replaceAppImage(current, path);
        if (!result.ok) return result;
        await promisify(execFile)(current, [marker]);
        return success(undefined);
      },
    });
    const updates = createApplicationUpdates({
      installedVersion: "0.0.2",
      backend,
    });
    installApplicationUpdates(
      ipcMain,
      updates,
      (event) => BrowserWindow.fromWebContents(event.sender),
      () => BrowserWindow.getAllWindows(),
    );
    const window = new BrowserWindow({
      width: 1200,
      height: 800,
      webPreferences: {
        contextIsolation: true,
        nodeIntegration: false,
        preload: process.argv.at(-1),
        backgroundThrottling: false,
      },
    });
    await window.loadFile(join(directory, "index.html"));
    await updates.check();
    await waitFor(window, "Update available");
    const layouts = [];
    for (const { id: theme } of MATERIALS)
      for (const mode of MODES)
        for (const width of [800, 1200]) {
          window.setContentSize(width, 600);
          await window.webContents.executeJavaScript(
            `document.documentElement.dataset.theme = ${JSON.stringify(theme)}; document.documentElement.dataset.mode = ${JSON.stringify(mode)}; new Promise(resolve => requestAnimationFrame(() => { document.getAnimations().filter(animation => animation.effect?.getComputedTiming().iterations !== Infinity).forEach(animation => animation.finish()); requestAnimationFrame(() => requestAnimationFrame(resolve)); }))`,
          );
          await new Promise((resolve) => {
            setTimeout(resolve, 150);
          });
          await window.webContents.executeJavaScript(
            "document.querySelector('.update-notice button').focus()",
          );
          for (const modifiers of [["shift"], []] as const) {
            window.webContents.sendInputEvent({
              type: "keyDown",
              keyCode: "Tab",
              modifiers: [...modifiers],
            });
            window.webContents.sendInputEvent({
              type: "keyUp",
              keyCode: "Tab",
              modifiers: [...modifiers],
            });
          }
          const layout = (await window.webContents.executeJavaScript(
            `(${measureUpdateStatusBar.toString()})()`,
          )) as ReturnType<typeof measureUpdateStatusBar>;
          layouts.push({ theme, mode, requestedWidth: width, ...layout });
          const capture = process.env.MUZO_UPDATE_CAPTURE_DIR;
          if (capture) {
            await window.webContents.executeJavaScript(
              "document.activeElement?.blur(); new Promise(resolve => requestAnimationFrame(() => requestAnimationFrame(resolve)))",
            );
            await mkdir(capture, { recursive: true });
            await writeFile(
              join(capture, `update-${theme}-${mode}-${width}.png`),
              (await window.webContents.capturePage()).toPNG(),
            );
          }
        }
    await click(window);
    await waitFor(window, "Update failed");
    const corruptionRejected =
      (await readFile(current, "utf8")) === "original executable";
    corrupt = false;
    await click(window);
    await waitFor(window, "Update available");
    await click(window);
    await waitFor(window, "Restart to install");
    if ((await readFile(current, "utf8")) !== "original executable")
      throw new Error("Update installed before restart");
    await click(window);
    await waitFor(window, "Restart to install");
    confirmed = true;
    await click(window);
    await new Promise((resolve) => {
      setTimeout(resolve, 150);
    });
    const installed = await readFile(marker, "utf8");
    await writeFile(
      join(directory, "result.json"),
      JSON.stringify({ corruptionRejected, installed, layouts }),
    );
    updates.dispose();
    window.destroy();
  } catch (error) {
    await writeFile(
      join(directory, "result.json"),
      JSON.stringify({ failure: String(error) }),
    );
    process.stderr.write(String(error));
    process.exitCode = 1;
  } finally {
    server.close();
    app.quit();
  }
});
