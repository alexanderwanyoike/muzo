import { readFileSync, statSync } from "node:fs";
import { fileURLToPath } from "node:url";
import { parse } from "smol-toml";
import { expect, it } from "vitest";

it("packages the native icons and licence notices from the Rust workspace", () => {
  const workspace = new URL("../../apps/desktop-rust/", import.meta.url);
  const packager = parse(
    readFileSync(new URL("Packager.toml", workspace), "utf8"),
  );
  expect(packager.icons).toEqual([
    "assets/icons/muzo.png",
    "assets/icons/muzo.ico",
    "assets/icons/muzo.icns",
  ]);
  expect(packager.resources).toContain("assets/licenses");
  for (const asset of [
    "assets/icons/muzo.png",
    "assets/icons/muzo.ico",
    "assets/icons/muzo.icns",
    "assets/licenses/Muzo-MIT.txt",
    "assets/licenses/manrope-OFL.txt",
    "assets/licenses/lucide.txt",
    "assets/licenses/ogg-BSD-3-Clause.txt",
    "assets/licenses/ruopus-MIT.txt",
  ])
    expect(statSync(fileURLToPath(new URL(asset, workspace))).isFile()).toBe(
      true,
    );
});
