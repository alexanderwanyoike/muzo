import { app, BrowserWindow } from "electron";
import { writeFileSync } from "node:fs";

const [page, resultPath] = process.argv.slice(-2);
if (!page || !resultPath) throw new Error("Seek fixture paths required");

function measureSeek() {
  const range = document.querySelector<HTMLInputElement>(
    'input[aria-label="Seek"]',
  );
  const shell = document.querySelector<HTMLElement>(
    ".now-playing-bar__progress-shell",
  );
  if (!range || !shell) throw new Error("Seek control missing");
  const rect = range.getBoundingClientRect();
  const style = getComputedStyle(shell);
  return {
    x: Math.round(rect.x + rect.width / 2),
    y: Math.round(rect.y + rect.height / 2),
    focused: document.activeElement === range,
    focusVisible: range.matches(":focus-visible"),
    outline: style.outlineStyle,
    outlineWidth: style.outlineWidth,
    value: range.valueAsNumber,
  };
}

void app.whenReady().then(async () => {
  try {
    const window = new BrowserWindow({
      width: 1280,
      height: 800,
      show: true,
      webPreferences: { backgroundThrottling: false },
    });
    const results = [];
    for (const mode of ["light", "dark"]) {
      await window.loadFile(page);
      await window.webContents
        .executeJavaScript(`new Promise((resolve, reject) => {
      const deadline = Date.now() + 5000;
      const check = () => {
        if (document.querySelector('input[aria-label="Seek"]')) resolve();
        else if (Date.now() > deadline) reject(new Error('Transport did not render'));
        else setTimeout(check, 20);
      };
      check();
    })`);
      await window.webContents.executeJavaScript(
        `document.documentElement.dataset.mode = ${JSON.stringify(mode)}`,
      );
      window.webContents.focus();
      window.webContents.sendInputEvent({
        type: "mouseDown",
        x: 20,
        y: 200,
        button: "left",
        clickCount: 1,
      });
      window.webContents.sendInputEvent({
        type: "mouseUp",
        x: 20,
        y: 200,
        button: "left",
        clickCount: 1,
      });
      const before = (await window.webContents.executeJavaScript(
        `(${measureSeek.toString()})()`,
      )) as ReturnType<typeof measureSeek>;
      window.webContents.sendInputEvent({
        type: "mouseDown",
        x: before.x,
        y: before.y,
        button: "left",
        clickCount: 1,
      });
      window.webContents.sendInputEvent({
        type: "mouseUp",
        x: before.x,
        y: before.y,
        button: "left",
        clickCount: 1,
      });
      await window.webContents.executeJavaScript(
        "new Promise(resolve => requestAnimationFrame(() => requestAnimationFrame(resolve)))",
      );
      const pointer: unknown = await window.webContents.executeJavaScript(
        `(${measureSeek.toString()})()`,
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
      await window.webContents.executeJavaScript(
        "new Promise(resolve => requestAnimationFrame(() => requestAnimationFrame(resolve)))",
      );
      const keyboard: unknown = await window.webContents.executeJavaScript(
        `(${measureSeek.toString()})()`,
      );
      window.webContents.sendInputEvent({ type: "keyDown", keyCode: "Right" });
      window.webContents.sendInputEvent({ type: "keyUp", keyCode: "Right" });
      await window.webContents.executeJavaScript(
        "new Promise(resolve => requestAnimationFrame(() => requestAnimationFrame(resolve)))",
      );
      const adjusted: unknown = await window.webContents.executeJavaScript(
        `(${measureSeek.toString()})()`,
      );
      results.push({ mode, pointer, keyboard, adjusted });
    }
    writeFileSync(resultPath, JSON.stringify(results));
    window.destroy();
  } catch (error) {
    process.stderr.write(String(error));
    process.exitCode = 1;
  } finally {
    app.quit();
  }
});
