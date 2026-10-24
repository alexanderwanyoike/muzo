import { app, BrowserWindow } from "electron";
import { writeFileSync } from "node:fs";
import { measureSettingsLayout } from "./measure-settings-layout";

const [page, resultPath] = process.argv.slice(-2);
if (!page || !resultPath) throw new Error("Settings fixture paths required");
void app.whenReady().then(async () => {
  try {
    const window = new BrowserWindow({ show: false, webPreferences: { backgroundThrottling: false } });
    const measurements = [];
    for (const width of [1280, 800]) {
      window.setContentSize(width, 600);
      for (const connected of [true, false]) {
        await window.loadFile(page, { query: { connected: String(connected) } });
        const layout: unknown = await window.webContents.executeJavaScript(`(${measureSettingsLayout.toString()})()`);
        measurements.push({ width, connected, layout });
      }
    }
    writeFileSync(resultPath, JSON.stringify(measurements));
    window.destroy();
  } catch (error) {
    process.stderr.write(String(error));
    process.exitCode = 1;
  } finally { app.quit(); }
});
