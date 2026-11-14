import { readFileSync } from "node:fs";
import { expect, it } from "vitest";

it("builds Muzo's native executable from the root desktop build command", () => {
  const root: unknown = JSON.parse(
    readFileSync(new URL("../../package.json", import.meta.url), "utf8"),
  );
  expect(root).toHaveProperty(
    ["scripts", "desktop:build"],
    "yarn workspace @muzo/desktop-rust build",
  );
});

it.each(["package", "make", "distribute"])(
  "packages the Rust desktop through the root %s command",
  (command) => {
    const root: unknown = JSON.parse(
      readFileSync(new URL("../../package.json", import.meta.url), "utf8"),
    );
    expect(root).toHaveProperty(
      ["scripts", `desktop:${command}`],
      "yarn workspace @muzo/distribution package:native",
    );
    expect(root).toHaveProperty(
      ["scripts", `desktop:legacy:${command}`],
      `yarn workspace @muzo/desktop ${command}`,
    );
  },
);
