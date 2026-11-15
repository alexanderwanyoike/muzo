import { execFile } from "node:child_process";
import { existsSync, readFileSync } from "node:fs";
import {
  copyFile,
  mkdir,
  mkdtemp,
  readFile,
  rm,
  writeFile,
} from "node:fs/promises";
import { tmpdir } from "node:os";
import { delimiter, join } from "node:path";
import { promisify } from "node:util";
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

it.skipIf(process.platform === "win32")(
  "cleans Muzo's Cargo artifacts and generated installers",
  async () => {
    const workspace = await mkdtemp(join(tmpdir(), "muzo-clean-"));
    try {
      const cargoArgs = join(workspace, "cargo-args");
      const installers = join(workspace, "out");
      const music = join(workspace, "music.mp3");
      await mkdir(installers);
      await Promise.all([
        copyFile(
          new URL("../../apps/desktop-rust/package.json", import.meta.url),
          join(workspace, "package.json"),
        ),
        writeFile(
          join(workspace, "cargo"),
          '#!/bin/sh\nprintf "%s\\n" "$@" > cargo-args\n',
          { mode: 0o700 },
        ),
        writeFile(join(installers, "Muzo.AppImage"), "fixture installer"),
        writeFile(music, "fixture track"),
      ]);

      await promisify(execFile)("yarn", ["run", "clean"], {
        cwd: workspace,
        env: {
          ...process.env,
          PATH: [workspace, process.env.PATH ?? ""].join(delimiter),
        },
      });

      expect(existsSync(installers)).toBe(false);
      expect((await readFile(cargoArgs, "utf8")).trim().split("\n")).toEqual([
        "clean",
        "--locked",
        "--package",
        "muzo-desktop",
      ]);
      expect(await readFile(music, "utf8")).toBe("fixture track");
    } finally {
      await rm(workspace, { recursive: true, force: true });
    }
  },
);
