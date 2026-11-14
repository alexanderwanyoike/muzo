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
