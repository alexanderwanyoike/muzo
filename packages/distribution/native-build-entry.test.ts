import { existsSync, readFileSync } from "node:fs";
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
  },
);

it("runs desktop development and verification from the Rust workspace", () => {
  const root: unknown = JSON.parse(
    readFileSync(new URL("../../package.json", import.meta.url), "utf8"),
  );
  expect(root).toHaveProperty(
    ["scripts", "desktop"],
    "yarn workspace @muzo/desktop-rust",
  );
  expect(root).toHaveProperty(
    ["scripts", "desktop:dev"],
    "yarn workspace @muzo/desktop-rust dev",
  );
  expect(root).toHaveProperty(
    ["scripts", "clean"],
    expect.stringContaining("yarn workspace @muzo/desktop-rust clean"),
  );
  for (const command of ["lint", "typecheck", "test"]) {
    expect(root).toHaveProperty(
      ["scripts", command],
      expect.stringContaining(
        `yarn workspace @muzo/desktop-rust native:${command}`,
      ),
    );
  }
});

it("keeps the Electron runtime out of a desktop checkout", () => {
  const root: unknown = JSON.parse(
    readFileSync(new URL("../../package.json", import.meta.url), "utf8"),
  );
  expect(root).not.toHaveProperty(["scripts", "desktop:vite"]);
  for (const command of [
    "desktop:legacy",
    "desktop:legacy:build",
    "desktop:legacy:package",
    "desktop:legacy:make",
    "desktop:legacy:distribute",
  ]) {
    expect(root).not.toHaveProperty(["scripts", command]);
  }
  const legacyWorkspace = new URL(
    "../../apps/desktop/package.json",
    import.meta.url,
  );
  expect(existsSync(legacyWorkspace)).toBe(false);
});
