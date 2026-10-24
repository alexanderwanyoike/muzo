// @vitest-environment node
import { readFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { expect, it } from "vitest";

const icons = join(dirname(fileURLToPath(import.meta.url)), "..", "assets", "icons");

it("ships a transparent high-resolution PNG and native Windows and macOS icon sizes", () => {
  const png = readFileSync(join(icons, "muzo.png"));
  expect(png.subarray(0, 8).toString("hex")).toBe("89504e470d0a1a0a");
  expect(png.readUInt32BE(16)).toBeGreaterThanOrEqual(1024);
  expect(png.readUInt32BE(20)).toBe(png.readUInt32BE(16));
  expect(png[25]).toBe(6);

  const ico = readFileSync(join(icons, "muzo.ico"));
  expect(ico.readUInt16LE(0)).toBe(0);
  expect(ico.readUInt16LE(2)).toBe(1);
  const sizes = [];
  for (let i = 0; i < ico.readUInt16LE(4); i += 1) {
    const entry = 6 + i * 16;
    sizes.push(ico[entry] || 256);
    const length = ico.readUInt32LE(entry + 8);
    const offset = ico.readUInt32LE(entry + 12);
    expect(offset + length).toBeLessThanOrEqual(ico.length);
    expect(length).toBeGreaterThan(0);
  }
  expect(sizes).toEqual(expect.arrayContaining([16, 24, 32, 48, 256]));

  const icns = readFileSync(join(icons, "muzo.icns"));
  expect(icns.subarray(0, 4).toString()).toBe("icns");
  expect(icns.readUInt32BE(4)).toBe(icns.length);
  const types = [];
  for (let offset = 8; offset < icns.length;) {
    types.push(icns.subarray(offset, offset + 4).toString());
    const length = icns.readUInt32BE(offset + 4);
    expect(length).toBeGreaterThan(8);
    offset += length;
    expect(offset).toBeLessThanOrEqual(icns.length);
  }
  expect(types).toEqual(expect.arrayContaining(["ic07", "ic08", "ic09", "ic10"]));
});
