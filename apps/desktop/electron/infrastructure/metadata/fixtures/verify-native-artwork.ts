import { app, nativeImage } from "electron";
import { readFileSync, writeFileSync } from "node:fs";
import { ElectronArtworkThumbnails } from "../electron-artwork-thumbnails";

const [picturePath, resultPath] = process.argv.slice(-2);
if (!picturePath || !resultPath) throw new Error("Native artwork fixture paths are required");

void app.whenReady().then(() => {
  try {
    const thumbnails = new ElectronArtworkThumbnails(nativeImage);
    const uri = thumbnails.create({ format: "image/png", data: readFileSync(picturePath) });
    if (!uri) throw new Error("Fixture cover could not be decoded");
    const pixels = Buffer.alloc(256 * 256 * 4);
    let seed = 123;
    for (let index = 0; index < pixels.length; index += 1) {
      seed ^= seed << 13;
      seed ^= seed >>> 17;
      seed ^= seed << 5;
      pixels[index] = seed & 255;
    }
    const detailedCover = thumbnails.create({ format: "image/png", data: nativeImage.createFromBitmap(pixels, { width: 256, height: 256 }).toPNG() });
    writeFileSync(resultPath, JSON.stringify({
      size: nativeImage.createFromDataURL(uri).getSize(),
      uri,
      noiseSize: detailedCover ? nativeImage.createFromDataURL(detailedCover).getSize() : null,
      noiseLength: detailedCover?.length ?? 0,
      invalid: thumbnails.create({ format: "image/png", data: Buffer.from("invalid") }),
      svg: thumbnails.create({ format: "image/svg+xml", data: Buffer.from("<svg/>") }),
    }));
  } finally { app.quit(); }
});
