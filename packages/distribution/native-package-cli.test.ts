import { execFile } from "node:child_process";
import { cp, mkdir, mkdtemp, readFile, rm, writeFile } from "node:fs/promises";
import { createRequire } from "node:module";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";
import { promisify } from "node:util";
import { expect, it } from "vitest";

const execute = promisify(execFile);

it("rejects version drift before starting Cargo or the native packager", async () => {
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
    const packager = join(fixture, "apps/desktop-rust/Packager.toml");
    const config = await readFile(packager, "utf8");
    await writeFile(
      packager,
      config.replace(/^version = ".*"$/m, 'version = "99.0.0"'),
    );
    const command = execute(
      process.execPath,
      [
        "--import",
        pathToFileURL(createRequire(import.meta.url).resolve("tsx/esm")).href,
        join(fixture, "packages/distribution/native-package-cli.ts"),
      ],
      {
        cwd: fileURLToPath(distribution),
        env: { ...process.env, PATH: "" },
      },
    );
    await expect(command).rejects.toMatchObject({
      stderr: expect.stringContaining("versions must match"),
    });
  } finally {
    await rm(fixture, { recursive: true, force: true });
  }
});
