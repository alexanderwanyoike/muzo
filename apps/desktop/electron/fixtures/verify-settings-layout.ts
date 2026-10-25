import { app, BrowserWindow } from "electron";
import { writeFileSync } from "node:fs";
import { ACCENTS, MATERIALS, MODES } from "../../src/appearance";
import { measureSplitterFocus } from "./measure-splitter-focus";
import { measureSettingsLayout } from "./measure-settings-layout";

const [page, resultPath] = process.argv.slice(-2);
if (!page || !resultPath) throw new Error("Settings fixture paths required");
void app.whenReady().then(async () => {
  try {
    const window = new BrowserWindow({
      show: true,
      webPreferences: { backgroundThrottling: false },
    });
    const measurements = [];
    for (const width of [1280, 800]) {
      window.setContentSize(width, 600);
      for (const { id: theme } of MATERIALS) {
        for (const mode of MODES) {
          for (const connected of [true, false]) {
            await window.loadFile(page, {
              query: { connected: String(connected), theme, mode },
            });
            const layout: unknown = await window.webContents.executeJavaScript(
              `(${measureSettingsLayout.toString()})()`,
            );
            await window.webContents.executeJavaScript(
              "document.querySelector('.sidebar-splitter').focus()",
            );
            window.webContents.focus();
            window.webContents.sendInputEvent({
              type: "keyDown",
              keyCode: "Tab",
              modifiers: ["shift"],
            });
            window.webContents.sendInputEvent({
              type: "keyUp",
              keyCode: "Tab",
              modifiers: ["shift"],
            });
            window.webContents.sendInputEvent({
              type: "keyDown",
              keyCode: "Tab",
            });
            window.webContents.sendInputEvent({
              type: "keyUp",
              keyCode: "Tab",
            });
            await window.webContents.executeJavaScript(
              "new Promise(resolve => requestAnimationFrame(() => requestAnimationFrame(resolve)))",
            );
            const splitterFocus: unknown =
              await window.webContents.executeJavaScript(
                `(${measureSplitterFocus.toString()})(${JSON.stringify(ACCENTS.filter(({ id }) => id.startsWith("glow-")).map(({ id }) => id))})`,
              );
            measurements.push({
              width,
              connected,
              theme,
              mode,
              layout,
              splitterFocus,
            });
          }
        }
      }
    }
    writeFileSync(resultPath, JSON.stringify(measurements));
    window.destroy();
  } catch (error) {
    process.stderr.write(String(error));
    process.exitCode = 1;
  } finally {
    app.quit();
  }
});
