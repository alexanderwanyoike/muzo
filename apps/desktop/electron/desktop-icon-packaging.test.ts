// @vitest-environment node
import { existsSync } from "node:fs";
import { createRequire } from "node:module";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { expect, it } from "vitest";

it("packages native icons on every platform and installs a Linux launcher icon", () => {
  const root = join(dirname(fileURLToPath(import.meta.url)), "..");
  const config = createRequire(import.meta.url)("../forge.config.cjs");
  expect(config.packagerConfig.icon).toBe(
    join(root, "assets", "icons", "muzo"),
  );
  for (const extension of ["png", "ico", "icns"]) {
    expect(existsSync(`${config.packagerConfig.icon}.${extension}`)).toBe(true);
    expect(
      config.packagerConfig.ignore.some((rule: RegExp) =>
        rule.test(`/assets/icons/muzo.${extension}`),
      ),
    ).toBe(false);
  }
  const deb = config.makers.find(
    (maker: { name: string }) => maker.name === "@electron-forge/maker-deb",
  );
  expect(deb.config.options.icon).toBe(`${config.packagerConfig.icon}.png`);
  expect(deb.config.options.productName).toBe("Muzo");
  expect(deb.config.options.genericName).toBe("Music player");
});
