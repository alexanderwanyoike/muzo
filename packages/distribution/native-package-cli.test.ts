import { execFile } from "node:child_process";
import {
  cp,
  mkdir,
  mkdtemp,
  readFile,
  rm,
  symlink,
  writeFile,
} from "node:fs/promises";
import { createRequire } from "node:module";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";
import { promisify } from "node:util";
import { expect, it } from "vitest";

const execute = promisify(execFile);

it.each([
  ["version drift", /versions must match/, "99.0.0"],
  ["an invalid configured Cargo target", /muzo-invalid-host-target/, undefined],
] as const)(
  "rejects %s before packaging stale binaries",
  async (_, error, version) => {
    if (process.platform !== "linux" && !version) return;
    const fixture = await mkdtemp(join(tmpdir(), "muzo-package-drift-"));
    try {
      const source = new URL("../../", import.meta.url);
      const workspaces = [
        "apps/desktop",
        "apps/desktop-rust",
        "packages/distribution",
      ];
      await Promise.all(
        workspaces.map((dir) => mkdir(join(fixture, dir), { recursive: true })),
      );
      const distribution = new URL(".", import.meta.url);
      const scripts = [
        "native-package-cli.ts",
        "release-context.ts",
        "release-versions.ts",
        "release-contract.ts",
        "distribution-result.ts",
      ];
      const paths = [
        "package.json",
        "apps/desktop/package.json",
        "apps/desktop-rust/package.json",
        "apps/desktop-rust/Cargo.toml",
        "apps/desktop-rust/Packager.toml",
        "packages/distribution/package.json",
      ];
      await Promise.all([
        ...paths.map((path) => cp(new URL(path, source), join(fixture, path))),
        ...scripts.map((file) =>
          cp(
            new URL(file, distribution),
            join(fixture, "packages/distribution", file),
          ),
        ),
      ]);
      const dependency = createRequire(import.meta.url).resolve("smol-toml");
      const nodeModules = join(fixture, "node_modules");
      await mkdir(nodeModules);
      await symlink(
        join(dependency, "..", ".."),
        join(nodeModules, "smol-toml"),
        "junction",
      );
      const packager = join(fixture, "apps/desktop-rust/Packager.toml");
      const config = await readFile(packager, "utf8");
      if (version) {
        await writeFile(
          packager,
          config.replace(/^version = ".*"$/m, `version = "${version}"`),
        );
      }
      if (process.platform !== "win32") {
        const cargo = join(fixture, "cargo");
        const fake = [
          "#!/bin/sh",
          'if [ "$1" = metadata ]; then',
          `  printf '{"target_directory":"/stale"}\\n'`,
          'elif [ "$1" = "-vV" ]; then',
          '  printf "host: x86_64-unknown-linux-gnu\\n"',
          "else",
          '  case " $* " in',
          '    *" --message-format=json-render-diagnostics "*) printf "error: muzo-invalid-host-target\\n" >&2; exit 1;;',
          "  esac",
          "fi",
        ];
        await writeFile(cargo, `${fake.join("\n")}\n`, { mode: 0o700 });
      }
      const command = execute(
        process.execPath,
        [
          "--import",
          pathToFileURL(createRequire(import.meta.url).resolve("tsx/esm")).href,
          join(fixture, "packages/distribution/native-package-cli.ts"),
        ],
        {
          cwd: fileURLToPath(distribution),
          env: {
            ...process.env,
            PATH: fixture,
            CARGO_BUILD_TARGET: "muzo-invalid-host-target",
          },
        },
      );
      await expect(command).rejects.toThrow(error);
    } finally {
      await rm(fixture, { recursive: true, force: true });
    }
  },
);
