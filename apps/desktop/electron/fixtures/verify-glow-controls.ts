import { app, BrowserWindow } from "electron";
import { writeFileSync } from "node:fs";
import { ACCENTS, MATERIALS } from "../../src/appearance";
import { measureGlowControls } from "./measure-glow-controls";

const [page, resultPath] = process.argv.slice(-2);
if (!page || !resultPath) throw new Error("Accent fixture paths required");
void app.whenReady().then(async () => {
  try {
    const window = new BrowserWindow({
      width: 1000,
      height: 700,
      show: true,
      webPreferences: { backgroundThrottling: false },
    });
    await window.loadFile(page);
    const result = (await window.webContents.executeJavaScript(
      `(${measureGlowControls.toString()})(${JSON.stringify(MATERIALS.map(({ id }) => id))}, ${JSON.stringify(ACCENTS.filter(({ id }) => id.startsWith("glow-")).map(({ id }) => id))})`,
    )) as { measurements: unknown[]; restored: unknown[] };
    window.webContents.debugger.attach("1.3");
    await window.webContents.debugger.sendCommand(
      "Emulation.setEmulatedMedia",
      { features: [{ name: "forced-colors", value: "active" }] },
    );
    const forced: unknown = await window.webContents.executeJavaScript(
      "new Promise(resolve => requestAnimationFrame(() => requestAnimationFrame(() => resolve({shadow:getComputedStyle(document.querySelector('.now-playing-bar__play')).boxShadow, filter:getComputedStyle(document.querySelector('.now-playing-bar__volume-slider')).filter}))))",
    );
    writeFileSync(resultPath, JSON.stringify({ ...result, forced }));
    window.destroy();
  } catch (error) {
    writeFileSync(resultPath, JSON.stringify({ failure: String(error) }));
    process.exitCode = 1;
  } finally {
    app.quit();
  }
});
