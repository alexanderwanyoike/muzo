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
import { basename, dirname, join, resolve } from "node:path";
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
const cargoHosts = {
  "linux-x64": /^x86_64-.*-linux-gnu$/u,
  "windows-x64": /^x86_64-.*-windows-(msvc|gnu)$/u,
  "macos-arm64": /^aarch64-apple-darwin$/u,
  "macos-x64": /^x86_64-apple-darwin$/u,
};

async function run(
  executable: string,
  args: string[],
  cwd: string = native,
): Promise<string> {
  const result = await execute(executable, args, {
    cwd,
    maxBuffer: 16 * 1024 * 1024,
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

async function buildExecutables(target: ReleaseTarget) {
  const { stdout } = await execute("cargo", ["-vV"], { cwd: native });
  const host = /^host: (.*)$/mu.exec(stdout)?.[1]?.trim();
  if (!host || !cargoHosts[target.id].test(host))
    return distributionFailure(
      "invalidRelease",
      "The Rust toolchain must match the requested installer architecture.",
    );

  const { stdout: messages, stderr } = await execute(
    "cargo",
    [
      "build",
      "--locked",
      "--release",
      "--message-format=json-render-diagnostics",
    ],
    { cwd: native, maxBuffer: 16 * 1024 * 1024 },
  );
  process.stderr.write(stderr);
  const executables = new Map<string, string>();
  for (const line of messages.trim().split(/\r?\n/u).filter(Boolean)) {
    const message: unknown = JSON.parse(line);
    if (
      message &&
      typeof message === "object" &&
      "reason" in message &&
      message.reason === "compiler-artifact" &&
      "executable" in message &&
      typeof message.executable === "string" &&
      "target" in message &&
      message.target &&
      typeof message.target === "object" &&
      "name" in message.target &&
      typeof message.target.name === "string"
    )
      executables.set(message.target.name, message.executable);
  }
  const application = executables.get("muzo-desktop");
  const namer = executables.get("name-installer");
  if (!application || !namer)
    return distributionFailure(
      "storage",
      "Cargo did not report the two desktop executables.",
    );
  return { ok: true as const, value: { application, namer } };
}

async function buildPayload(
  target: ReleaseTarget,
  version: string,
  executable: string,
  stage: string,
) {
  if (target.platform === "windows") {
    const payload = join(stage, "payload");
    const resources = join(payload, "resources");
    await mkdir(resources, { recursive: true });
    await cp(executable, join(payload, "muzo.exe"));
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
    dirname(executable),
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
    const executables = await buildExecutables(target);
    if (!executables.ok) return executables;
    const builtTarget = await execute(executables.value.namer, ["--target-id"]);
    if (builtTarget.stdout.trim() !== target.id)
      return distributionFailure(
        "invalidRelease",
        "Cargo built a different installer target.",
      );
    await mkdir(output, { recursive: true });
    const stage = await mkdtemp(join(output, ".muzo-package-"));
    try {
      await buildPayload(
        target,
        version.value,
        executables.value.application,
        stage,
      );
      const renamed = await run(executables.value.namer, [stage]);
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
