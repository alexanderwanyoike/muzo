// @vitest-environment node

import { execFileSync } from "node:child_process";
import { existsSync, mkdtempSync, readFileSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

import { describe, expect, it } from "vitest";

import { buildElectronMain } from "./build-electron-main";

const electronDir = dirname(fileURLToPath(import.meta.url));
const migrationsDir = join(
  electronDir,
  "infrastructure",
  "sqlite",
  "migrations",
);

describe("buildElectronMain", () => {
  it("builds the Electron main process and runtime assets", async () => {
    const root = mkdtempSync(join(tmpdir(), "muzo-electron-build-"));
    const entryPoint = join(root, "main.ts");
    const dependency = join(root, "dynamic-require.cjs");
    const preload = join(root, "preload.cjs");
    const outdir = join(root, "dist-electron");
    writeFileSync(
      entryPoint,
      [
        "import { dirname } from 'node:path';",
        "import { fileURLToPath } from 'node:url';",
        "import dependency from './dynamic-require.cjs';",
        "console.log(dirname(fileURLToPath(import.meta.url)));",
        "console.log(dependency());",
      ].join("\n"),
    );
    writeFileSync(
      dependency,
      "module.exports = () => require('node:os').platform();",
    );
    writeFileSync(preload, "module.exports = {};");

    await buildElectronMain({
      entryPoint,
      migrationsDir,
      preload,
      outdir,
    });

    expect(existsSync(join(outdir, "main.cjs"))).toBe(true);
    expect(readFileSync(join(outdir, "preload.cjs"), "utf8")).toBe(
      "module.exports = {};",
    );
    expect(existsSync(join(outdir, "sql-wasm.wasm"))).toBe(true);
    expect(execFileSync("node", [join(outdir, "main.cjs")], { encoding: "utf8" }))
      .toContain(outdir);
  });

  it("bundles the real Electron container without constructor name coupling", async () => {
    const root = mkdtempSync(join(tmpdir(), "muzo-electron-container-build-"));
    const entryPoint = join(root, "main.ts");
    const preload = join(root, "preload.cjs");
    const outdir = join(root, "dist-electron");
    const dbPath = join(root, "muzo.sqlite");
    const containerModule = join(
      electronDir,
      "composition",
      "electron-container.ts",
    );
    writeFileSync(
      entryPoint,
      [
        `import { createElectronContainer } from ${JSON.stringify(containerModule)};`,
        "async function main() {",
        `  const container = createElectronContainer({ dbPath: ${JSON.stringify(dbPath)} });`,
        "  await container.resolve('migrateDatabase')();",
        "  await container.resolve('watchFilesystemLibraries')();",
        "  await container.resolve('commandDispatcher').handleElectronCommand('list_libraries');",
        "  console.log('container-ok');",
        "}",
        "main().catch((error) => {",
        "  console.error(error);",
        "  process.exitCode = 1;",
        "});",
      ].join("\n"),
    );
    writeFileSync(preload, "module.exports = {};");

    await buildElectronMain({
      entryPoint,
      migrationsDir,
      preload,
      outdir,
    });

    expect(execFileSync("node", [join(outdir, "main.cjs")], { encoding: "utf8" }))
      .toContain("container-ok");
  });
});
