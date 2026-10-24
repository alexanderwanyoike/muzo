const { app, nativeImage } = require("electron");
const { writeFileSync } = require("node:fs");
const { join } = require("node:path");

const size = 48;
const pixels = Buffer.alloc(size * size * 4);
const heights = [8, 16, 24, 12, 4, 12, 24, 16, 8];
for (let y = 0; y < size; y += 1) {
  for (let x = 0; x < size; x += 1) {
    const px = x + 0.5;
    const py = y + 0.5;
    const cornerX = Math.max(12, Math.min(36, px));
    const cornerY = Math.max(12, Math.min(36, py));
    if (Math.hypot(px - cornerX, py - cornerY) > 10) continue;
    const blue = heights.some((height, index) => {
      const centreX = 7 + index * 4;
      const start = size / 2 - height / 2 + 1;
      const end = size / 2 + height / 2 - 1;
      return Math.hypot(px - centreX, py - Math.max(start, Math.min(end, py))) <= 1;
    });
    const offset = (y * size + x) * 4;
    pixels[offset] = blue ? 0xd4 : 0x16;
    pixels[offset + 1] = blue ? 0x78 : 0x12;
    pixels[offset + 2] = blue ? 0x34 : 0x10;
    pixels[offset + 3] = 255;
  }
}
const image = nativeImage.createFromBitmap(pixels, { width: size, height: size });
writeFileSync(join(__dirname, "..", "assets", "icons", "muzo-window.png"), image.toPNG());
app.exit(0);
