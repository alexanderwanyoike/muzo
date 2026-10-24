const { app, nativeImage } = require("electron");
const { writeFileSync } = require("node:fs");
const { join } = require("node:path");

const panelSize = 24;
const scale = 2;
const size = panelSize * scale;
const tileInset = scale;
const cornerRadius = 5 * scale;
const innerStart = tileInset + cornerRadius;
const innerEnd = size - innerStart;
const firstBarCentre = 3.5 * scale;
const barStep = 2 * scale;
const barRadius = scale / 2;
const black = [0x16, 0x12, 0x10, 255];
const blueFill = [0xd4, 0x78, 0x34, 255];
const pixels = Buffer.alloc(size * size * 4);
const heights = [4, 8, 12, 6, 2, 6, 12, 8, 4].map((height) => height * scale);
for (let y = 0; y < size; y += 1) {
  for (let x = 0; x < size; x += 1) {
    const px = x + 0.5;
    const py = y + 0.5;
    const cornerX = Math.max(innerStart, Math.min(innerEnd, px));
    const cornerY = Math.max(innerStart, Math.min(innerEnd, py));
    if (Math.hypot(px - cornerX, py - cornerY) <= cornerRadius) {
      const blue = heights.some((height, index) => {
        const centreX = firstBarCentre + index * barStep;
        const start = size / 2 - height / 2 + barRadius;
        const end = size / 2 + height / 2 - barRadius;
        return (
          Math.hypot(px - centreX, py - Math.max(start, Math.min(end, py))) <=
          barRadius
        );
      });
      const offset = (y * size + x) * 4;
      pixels.set(blue ? blueFill : black, offset);
    }
  }
}
const image = nativeImage.createFromBitmap(pixels, {
  width: size,
  height: size,
});
writeFileSync(
  join(__dirname, "assets", "icons", "muzo-window.png"),
  image.toPNG(),
);
app.exit(0);
