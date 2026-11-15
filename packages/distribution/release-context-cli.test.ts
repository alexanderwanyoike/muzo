import { execFile } from "node:child_process";
import { cp, mkdir, mkdtemp, rm, symlink } from "node:fs/promises";
import { createRequire } from "node:module";
import { tmpdir } from "node:os";
import { dirname, join } from "node:path";
import { pathToFileURL } from "node:url";
import { promisify } from "node:util";
import { expect, it } from "vitest";

const execute = promisify(execFile);
const require = createRequire(import.meta.url);

it("resolves the native release context without an Electron workspace", async () => {
  const checkout = await mkdtemp(join(tmpdir(), "muzo-native-context-"));
  try {
    const root = new URL("../../", import.meta.url);
    const files = [
      "package.json",
      "apps/desktop-rust/package.json",
      "apps/desktop-rust/Cargo.toml",
      "apps/desktop-rust/Packager.toml",
      "packages/distribution/package.json",
      "packages/distribution/distribution-result.ts",
      "packages/distribution/release-contract.ts",
      "packages/distribution/release-context.ts",
      "packages/distribution/release-context-cli.ts",
      "packages/distribution/release-versions.ts",
    ];
    await Promise.all(
      files.map(async (file) => {
        const destination = join(checkout, file);
        await mkdir(dirname(destination), { recursive: true });
        await cp(new URL(file, root), destination);
      }),
    );
    const dependencies = join(checkout, "node_modules");
    await mkdir(dependencies);
    await symlink(
      dirname(dirname(require.resolve("smol-toml"))),
      join(dependencies, "smol-toml"),
      "junction",
    );

    const sourceCommit = "a".repeat(40);
    const { stdout } = await execute(
      process.execPath,
      [
        "--import",
        pathToFileURL(require.resolve("tsx/esm")).href,
        join(checkout, "packages/distribution/release-context-cli.ts"),
        "--native",
        "--source-commit",
        sourceCommit,
      ],
      {
        cwd: checkout,
        env: { ...process.env, GITHUB_REF_TYPE: "branch" },
      },
    );
    const context: unknown = JSON.parse(stdout);
    const version = /^v(?<version>\d+\.\d+\.\d+)$/u;
    expect(context).toHaveProperty("tag", expect.stringMatching(version));
    expect(context).toHaveProperty("sourceCommit", sourceCommit);
  } finally {
    await rm(checkout, { recursive: true, force: true });
  }
});
