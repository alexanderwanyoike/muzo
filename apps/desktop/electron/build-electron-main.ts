import { cp, mkdir, copyFile } from "node:fs/promises";
import { createRequire } from "node:module";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

import { build } from "esbuild";

export interface BuildElectronMainOptions {
  entryPoint: string;
  migrationsDir: string;
  preload: string;
  outdir: string;
}

export async function buildElectronMain(
  options: BuildElectronMainOptions = defaultOptions(),
): Promise<void> {
  await mkdir(options.outdir, { recursive: true });
  await build({
    entryPoints: [options.entryPoint],
    outfile: join(options.outdir, "main.cjs"),
    bundle: true,
    platform: "node",
    format: "cjs",
    target: "node20",
    sourcemap: true,
    define: {
      "import.meta.url": "importMetaUrl",
    },
    banner: {
      js: 'const importMetaUrl = require("node:url").pathToFileURL(__filename).href;',
    },
    external: ["electron"],
  });
  await copyFile(options.preload, join(options.outdir, "preload.cjs"));
  await cp(options.migrationsDir, join(options.outdir, "migrations"), {
    recursive: true,
  });
  const require = createRequire(import.meta.url);
  await copyFile(
    require.resolve("sql.js/dist/sql-wasm.wasm"),
    join(options.outdir, "sql-wasm.wasm"),
  );
}

function defaultOptions(): BuildElectronMainOptions {
  const electronDir = dirname(fileURLToPath(import.meta.url));
  return {
    entryPoint: join(electronDir, "main.ts"),
    migrationsDir: join(electronDir, "infrastructure", "sqlite", "migrations"),
    preload: join(electronDir, "preload.cjs"),
    outdir: join(electronDir, "..", "dist-electron"),
  };
}

if (process.argv[1] === fileURLToPath(import.meta.url)) {
  await buildElectronMain();
}
