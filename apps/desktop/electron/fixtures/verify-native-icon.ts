import { execFileSync } from "node:child_process";
import { app, BrowserWindow, nativeImage } from "electron";
import { basename, dirname } from "node:path";
import { writeFileSync } from "node:fs";
import { configureDesktopIdentity } from "../desktop-identity";
import { configureDesktopIcon } from "../desktop-icon";

const [electronDir, resultPath] = process.argv.slice(-2);
if (!electronDir || !resultPath) throw new Error("Icon fixture paths required");

app.setPath("appData", dirname(resultPath));
configureDesktopIdentity(app);

void app.whenReady().then(async () => {
  try {
    const path = configureDesktopIcon(app, electronDir, process.platform);
    const image = nativeImage.createFromPath(path);
    const sizes = [16, 32, 256].map((size) =>
      image.resize({ width: size, height: size }).getSize(),
    );
    const window = new BrowserWindow({ show: false, icon: path });
    let panelIcon = null;
    if (process.platform === "linux") {
      await new Promise<void>((resolve) => { setImmediate(resolve); });
      const handle = window.getNativeWindowHandle();
      const id = handle.length >= 8 ? handle.readBigUInt64LE().toString() : String(handle.readUInt32LE());
      const property = execFileSync("xprop", ["-id", id, "-len", "16777216", "-f", "_NET_WM_ICON", "32c", "_NET_WM_ICON"], { encoding: "utf8", maxBuffer: 40 * 1024 * 1024 });
      const values = property.slice(property.indexOf("=") + 1).match(/\d+/g)?.map(Number) ?? [];
      const [width, height] = values;
      if (!width || !height || width > 2048 || height > 2048 || values.length < 2 + width * height) throw new Error("Incomplete or oversized window icon property");
      const bitmap = Buffer.alloc(width * height * 4);
      for (let i = 0; i < width * height; i += 1) bitmap.writeUInt32LE(values[i + 2], i * 4);
      const advertised = nativeImage.createFromBitmap(bitmap, { width, height });
      const bars = [24, 32].map((size) => {
        const pixels = advertised.resize({ width: size, height: size }).toBitmap();
        let runs = 0;
        let previous = false;
        for (let x = 0; x < size; x += 1) {
          const offset = (Math.floor(size / 2) * size + x) * 4;
          const blue = pixels[offset] > 100 && pixels[offset] - pixels[offset + 2] > 40;
          if (blue && !previous) runs += 1;
          previous = blue;
        }
        return { size, runs };
      });
      panelIcon = { width, height, bars };
    }
    writeFileSync(
      resultPath,
      JSON.stringify({
        identity: {
          name: app.getName(),
          profile: basename(app.getPath("userData")),
          session: basename(app.getPath("sessionData")),
        },
        panelIcon,
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
