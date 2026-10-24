import { app, BrowserWindow, nativeImage } from "electron";
import { basename, dirname } from "node:path";
import { writeFileSync } from "node:fs";
import { configureDesktopIdentity } from "../desktop-identity";
import { configureDesktopIcon } from "../desktop-icon";

const [electronDir, resultPath] = process.argv.slice(-2);
if (!electronDir || !resultPath) throw new Error("Icon fixture paths required");

app.setPath("appData", dirname(resultPath));
configureDesktopIdentity(app);

void app.whenReady().then(() => {
  try {
    const path = configureDesktopIcon(app, electronDir, process.platform);
    const image = nativeImage.createFromPath(path);
    const sizes = [16, 32, 256].map((size) =>
      image.resize({ width: size, height: size }).getSize(),
    );
    const window = new BrowserWindow({ show: false, icon: path });
    writeFileSync(
      resultPath,
      JSON.stringify({
        identity: {
          name: app.getName(),
          profile: basename(app.getPath("userData")),
          session: basename(app.getPath("sessionData")),
        },
        decoded: !image.isEmpty(),
        sizes,
        windowCreated: !window.isDestroyed(),
      }),
    );
    window.destroy();
  } finally {
    app.quit();
  }
});
