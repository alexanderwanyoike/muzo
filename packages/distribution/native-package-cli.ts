import { execFile } from "node:child_process";
import {
  cp,
  mkdir,
  mkdtemp,
  rename,
  rm,
  stat,
  symlink,
} from "node:fs/promises";
import { basename, join, resolve } from "node:path";
import { createRequire } from "node:module";
import { fileURLToPath } from "node:url";
import { parseArgs, promisify } from "node:util";
import {
  RELEASE_TARGETS,
  releaseAssetName,
  type ReleaseTarget,
} from "./release-contract";
import { releaseVersion } from "./release-context";
import { readReleaseVersions } from "./release-versions";
import {
  distributionFailure,
  type DistributionResult,
} from "./distribution-result";

const execute = promisify(execFile);
const native = fileURLToPath(
  new URL("../../apps/desktop-rust/", import.meta.url),
);
const legacy = fileURLToPath(new URL("../../apps/desktop/", import.meta.url));
const packageFiles = {
  linux: ["latest-linux.yml"],
  windows: ["latest.yml"],
  macos: [],
} as const;
const hostPlatforms = new Map<NodeJS.Platform, ReleaseTarget["platform"]>([
  ["linux", "linux"],
  ["darwin", "macos"],
  ["win32", "windows"],
]);

async function run(
  executable: string,
  args: string[],
  cwd: string = native,
): Promise<string> {
  const result = await execute(executable, args, {
    cwd,
    env: {
      ...process.env,
      APPIMAGE_EXTRACT_AND_RUN: "1",
      CSC_IDENTITY_AUTO_DISCOVERY: "false",
    },
  });
  process.stdout.write(result.stdout);
  process.stderr.write(result.stderr);
  return result.stdout;
}

async function buildPayload(
  target: ReleaseTarget,
  version: string,
  binaries: string,
  stage: string,
) {
  if (target.platform === "windows") {
    const payload = join(stage, "payload");
    const resources = join(payload, "resources");
    await mkdir(resources, { recursive: true });
    await cp(join(binaries, "muzo-desktop.exe"), join(payload, "muzo.exe"));
    await cp(join(legacy, "assets/licenses"), join(resources, "licenses"), {
      recursive: true,
    });
    const require = createRequire(join(legacy, "package.json"));
    await run(
      process.execPath,
      [
        require.resolve("electron-builder/cli.js"),
        "--prepackaged",
        payload,
        "--win",
        "nsis",
        "--x64",
        "--config",
        "electron-builder.config.json",
        `--config.extraMetadata.version=${version}`,
        `--config.directories.output=${stage}`,
        "--publish",
        "never",
      ],
      legacy,
    );
    return;
  }

  await run("cargo-packager", [
    "--config",
    "Packager.toml",
    "--binaries-dir",
    binaries,
    "--out-dir",
    stage,
    "--formats",
    target.platform === "linux" ? "appimage" : "app",
  ]);
  if (target.platform === "macos") {
    const contents = join(stage, "dmg");
    await mkdir(contents);
    await run("ditto", [join(stage, "Muzo.app"), join(contents, "Muzo.app")]);
    await symlink("/Applications", join(contents, "Applications"));
    await run("hdiutil", [
      "create",
      "-format",
      "UDZO",
      "-volname",
      "Muzo",
      "-srcfolder",
      contents,
      join(stage, "Muzo.dmg"),
    ]);
  }
}

async function packageNative(): Promise<DistributionResult<string>> {
  try {
    const { values } = parseArgs({
      options: { target: { type: "string" }, output: { type: "string" } },
    });
    const platform = hostPlatforms.get(process.platform);
    const target = RELEASE_TARGETS.find(
      (candidate) =>
        candidate.platform === platform && candidate.arch === process.arch,
    );
    if (!target || (values.target !== undefined && target.id !== values.target))
      return distributionFailure(
        "invalidRelease",
        "Build the requested Muzo installer on its native operating system and architecture.",
      );

    const version = releaseVersion(await readReleaseVersions(true));
    if (!version.ok) return version;

    const output = values.output
      ? resolve(process.env.INIT_CWD ?? process.cwd(), values.output)
      : join(native, "out/rust-distribution");
    const { stdout } = await execute(
      "cargo",
      ["metadata", "--locked", "--no-deps", "--format-version", "1"],
      { cwd: native },
    );
    const metadata: unknown = JSON.parse(stdout);
    if (
      !metadata ||
      typeof metadata !== "object" ||
      !("target_directory" in metadata) ||
      typeof metadata.target_directory !== "string"
    )
      return distributionFailure(
        "storage",
        "Could not locate Cargo’s target directory.",
      );
    const binaries = join(metadata.target_directory, "release");
    await run("cargo", ["build", "--locked", "--release"]);
    await mkdir(output, { recursive: true });
    const stage = await mkdtemp(join(output, ".muzo-package-"));
    try {
      await buildPayload(target, version.value, binaries, stage);
      const suffix = process.platform === "win32" ? ".exe" : "";
      const renamed = await run(join(binaries, `name-installer${suffix}`), [
        stage,
      ]);
      const name = basename(renamed.trim());
      if (
        name !== releaseAssetName(target.id, version.value) ||
        !(await stat(join(stage, name))).isFile()
      )
        return distributionFailure(
          "storage",
          "The expected native installer was not produced.",
        );
      await Promise.all(
        [...packageFiles[target.platform], name].map((file) =>
          rename(join(stage, file), join(output, file)),
        ),
      );
      return { ok: true, value: join(output, name) };
    } finally {
      await rm(stage, { recursive: true, force: true });
    }
  } catch (error) {
    if (
      error &&
      typeof error === "object" &&
      "stderr" in error &&
      typeof error.stderr === "string"
    )
      process.stderr.write(error.stderr);
    return distributionFailure(
      "storage",
      error instanceof Error
        ? error.message
        : "Native installer packaging failed.",
    );
  }
}

const result = await packageNative();
if (result.ok) process.stdout.write(`Installer ready: ${result.value}\n`);
else {
  process.stderr.write(`${result.error.message}\n`);
  process.exitCode = 1;
}
