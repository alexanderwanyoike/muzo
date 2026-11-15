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
import { dirname, join } from "node:path";
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
      const workspaces = ["apps/desktop-rust", "packages/distribution"];
      await Promise.all(
        workspaces.map((dir) => mkdir(join(fixture, dir), { recursive: true })),
      );
      const distribution = new URL(".", import.meta.url);
      const scripts = [
        "native-package-cli.ts",
        "native-package-output.ts",
        "release-context.ts",
        "release-versions.ts",
        "release-contract.ts",
        "distribution-result.ts",
      ];
      const paths = [
        "package.json",
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
          "  printf '%s\\n' \"$MUZO_FIXTURE_CARGO_METADATA\"",
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
            MUZO_FIXTURE_CARGO_METADATA: JSON.stringify({
              target_directory: join(fixture, "stale"),
            }),
          },
        },
      );
      await expect(command).rejects.toThrow(error);
    } finally {
      await rm(fixture, { recursive: true, force: true });
    }
  },
);

it.skipIf(process.platform !== "linux" || process.arch !== "x64")(
  "packages its own release binary while reusing the external Cargo build cache",
  async () => {
    const fixture = await mkdtemp(join(tmpdir(), "muzo-external-build-"));
    try {
      const cache = join(fixture, "external build");
      const configPath = join(fixture, "packager-config");
      const output = join(fixture, "installers");
      const script = (name: string, contents: string) =>
        writeFile(join(fixture, name), `#!${process.execPath}\n${contents}`, {
          mode: 0o700,
        });
      await Promise.all([
        script(
          "cargo",
          `const fs = require("node:fs");
const { dirname, join } = require("node:path");
const args = process.argv.slice(2);
const cache = process.env.MUZO_FIXTURE_CACHE;
if (args[0] === "-vV") {
  console.log("host: x86_64-unknown-linux-gnu");
} else if (args[0] === "metadata") {
  console.log(JSON.stringify({ target_directory: cache, build_directory: cache }));
} else {
  const index = args.indexOf("--target-dir");
  const binaries = join(index < 0 ? cache : args[index + 1], "release");
  fs.mkdirSync(binaries, { recursive: true });
  fs.writeFileSync(join(binaries, "muzo-desktop"), "candidate executable");
  fs.writeFileSync(join(binaries, "name-installer"), "#!/bin/sh\\nprintf 'linux-x64\\\\n'\\n", { mode: 0o700 });
  fs.writeFileSync(join(cache, "requested-build-cache"), process.env.CARGO_BUILD_BUILD_DIR ?? "");
  fs.writeFileSync(join(cache, "executable-staging-directory"), dirname(binaries));
  for (const name of ["muzo-desktop", "name-installer"]) {
    console.log(JSON.stringify({ reason: "compiler-artifact", executable: join(binaries, name), target: { name } }));
  }
}
`,
        ),
        script(
          "cargo-packager",
          `const fs = require("node:fs");
const { join } = require("node:path");
const config = JSON.parse(process.argv[3]);
const cache = process.env.MUZO_FIXTURE_CACHE;
const competingBuild = join(cache, "release");
fs.mkdirSync(competingBuild, { recursive: true });
fs.writeFileSync(join(competingBuild, "muzo-desktop"), "another checkout");
const packaged = fs.readFileSync(join(config["binaries-dir"], "muzo-desktop"), "utf8");
fs.writeFileSync(process.env.MUZO_FIXTURE_PACKAGER_CONFIG, JSON.stringify({ config, packaged }));
console.error("packager fixture reached");
process.exit(43);
`,
        ),
        mkdir(cache),
      ]);

      await expect(
        execute(
          process.execPath,
          [
            "--import",
            pathToFileURL(createRequire(import.meta.url).resolve("tsx/esm"))
              .href,
            fileURLToPath(new URL("native-package-cli.ts", import.meta.url)),
            "--output",
            output,
          ],
          {
            env: {
              ...process.env,
              PATH: fixture,
              MUZO_FIXTURE_CACHE: cache,
              MUZO_FIXTURE_PACKAGER_CONFIG: configPath,
            },
          },
        ),
      ).rejects.toThrow("packager fixture reached");
      const result: unknown = JSON.parse(await readFile(configPath, "utf8"));
      expect(result).toHaveProperty("packaged", "candidate executable");
      expect(result).toHaveProperty("config.appimage.libs", [
        "libxkbcommon-x11.so.0",
      ]);
      expect(await readFile(join(cache, "requested-build-cache"), "utf8")).toBe(
        cache,
      );
      const artifacts = await readFile(
        join(cache, "executable-staging-directory"),
        "utf8",
      );
      expect(
        dirname(artifacts),
        "keep executable artifacts off the installer output mount",
      ).toBe(cache);
    } finally {
      await rm(fixture, { recursive: true, force: true });
    }
  },
);
