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

it("keeps the existing Windows installer identity in the native packaging configuration", () => {
  const workspace = new URL("../../apps/desktop-rust/", import.meta.url);
  const config: unknown = JSON.parse(
    readFileSync(new URL("windows-installer.json", workspace), "utf8"),
  );
  expect(config).toMatchObject({
    appId: "com.muzo.desktop",
    productName: "Muzo",
    extraMetadata: { name: "@muzo/desktop", author: "Muzo" },
    win: {
      executableName: "muzo",
      icon: "assets/icons/muzo.ico",
      artifactName: "Muzo-${version}-windows-${arch}-setup.${ext}",
    },
    nsis: {
      oneClick: false,
      perMachine: false,
      allowToChangeInstallationDirectory: true,
    },
  });
  const distribution: unknown = JSON.parse(
    readFileSync(new URL("package.json", import.meta.url), "utf8"),
  );
  expect(distribution).toHaveProperty(["devDependencies", "electron-builder"]);
});
