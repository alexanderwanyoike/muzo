import assert from "node:assert/strict";
import { readFile, writeFile } from "node:fs/promises";
import { join } from "node:path";
import { pathToFileURL } from "node:url";

const jsonVersion = /(^  "version": ")(\d+\.\d+\.\d+)(")/mu;
const tomlVersion = /(^version = ")(\d+\.\d+\.\d+)(")/mu;
const lockedDesktop =
  /(\[\[package\]\]\r?\nname = "muzo-desktop"\r?\nversion = ")(\d+\.\d+\.\d+)(")/u;
const versionFiles = new Map([
  ["package.json", jsonVersion],
  ["apps/desktop-rust/package.json", jsonVersion],
  ["apps/desktop-rust/Cargo.toml", tomlVersion],
  ["apps/desktop-rust/Packager.toml", tomlVersion],
  ["apps/desktop-rust/Cargo.lock", lockedDesktop],
]);

function components(version) {
  assert.match(version, /^\d+\.\d+\.\d+$/u);
  return version.split(".").map(Number);
}

export async function prepareUpdaterFixtureVersion(root, legacy) {
  const current = JSON.parse(
    await readFile(join(root, "package.json"), "utf8"),
  ).version;
  const candidate = components(current);
  const previous = components(legacy);
  const difference =
    candidate
      .map((part, index) => part - previous[index])
      .find((part) => part !== 0) ?? 0;
  assert(
    difference >= 0,
    "The checkout predates the published updater fixture",
  );
  if (difference > 0) return current;

  const [major, minor, patch] = previous;
  const successor = [major, minor, patch + 1].join(".");
  const replacements = await Promise.all(
    Array.from(versionFiles, async ([file, pattern]) => {
      const path = join(root, file);
      const content = await readFile(path, "utf8");
      assert.equal(
        pattern.exec(content)?.[2],
        current,
        file + " has a different version",
      );
      return {
        path,
        content: content.replace(
          pattern,
          (_, prefix, _version, suffix) => prefix + successor + suffix,
        ),
      };
    }),
  );
  await Promise.all(
    replacements.map(({ path, content }) => writeFile(path, content)),
  );
  return successor;
}

if (
  process.argv[1] &&
  import.meta.url === pathToFileURL(process.argv[1]).href
) {
  const version = await prepareUpdaterFixtureVersion(
    process.cwd(),
    process.argv[2],
  );
  console.log("Updater candidate v" + version);
}
