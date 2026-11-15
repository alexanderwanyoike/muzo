import assert from "node:assert/strict";
import { mkdir, mkdtemp, readFile, rm, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { test } from "node:test";

import { prepareUpdaterFixtureVersion } from "./windows-updater-fixture-version.mjs";

test("builds a successor to the released version without changing dependencies", async (context) => {
  const root = await mkdtemp(join(tmpdir(), "muzo-updater-version-"));
  context.after(() => rm(root, { recursive: true, force: true }));
  await mkdir(join(root, "apps/desktop-rust"), { recursive: true });
  const versions = new Map([
    ["package.json", '{\n  "name": "muzo",\n  "version": "0.0.6"\n}\n'],
    [
      "apps/desktop-rust/package.json",
      '{\n  "name": "@muzo/desktop-rust",\n  "version": "0.0.6"\n}\n',
    ],
    [
      "apps/desktop-rust/Cargo.toml",
      '[package]\nname = "muzo-desktop"\nversion = "0.0.6"\n',
    ],
    [
      "apps/desktop-rust/Packager.toml",
      'name = "muzo-desktop"\nversion = "0.0.6"\n',
    ],
    [
      "apps/desktop-rust/Cargo.lock",
      '[[package]]\nname = "muzo-desktop"\nversion = "0.0.6"\n\n[[package]]\nname = "dependency"\nversion = "0.0.6"\n',
    ],
  ]);
  await Promise.all(
    Array.from(versions, ([file, body]) => writeFile(join(root, file), body)),
  );

  assert.equal(await prepareUpdaterFixtureVersion(root, "0.0.6"), "0.0.7");
  for (const [file, body] of versions) {
    assert.equal(
      await readFile(join(root, file), "utf8"),
      body.replace('"0.0.6"', '"0.0.7"'),
    );
  }
  assert.equal(await prepareUpdaterFixtureVersion(root, "0.0.6"), "0.0.7");
});
