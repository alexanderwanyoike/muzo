import { execFile } from "node:child_process";
import { readFile, stat } from "node:fs/promises";
import { createRequire } from "node:module";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { parseArgs, promisify } from "node:util";
import {
  releaseAssetName,
  isStableReleaseVersion,
  type DistributionResult,
} from "@muzo/distribution";
import { distributionPlan } from "./distribution-plan";

async function makeDistribution(): Promise<DistributionResult<string>> {
  try {
    const { values } = parseArgs({ options: { target: { type: "string" } } });
    const plan = distributionPlan(values.target ?? "", process.platform);
    if (!plan.ok) return plan;
    const root = join(dirname(fileURLToPath(import.meta.url)), "../..");
    const metadata: unknown = JSON.parse(
      await readFile(join(root, "package.json"), "utf8"),
    );
    if (
      !metadata ||
      typeof metadata !== "object" ||
      !("version" in metadata) ||
      !isStableReleaseVersion(metadata.version)
    )
      return {
        ok: false,
        error: {
          kind: "invalidRelease",
          message: "The desktop package must have a stable release version.",
        },
      };
    const require = createRequire(import.meta.url);
    for (const [entry, args] of [
      [
        require.resolve("@electron-forge/cli/dist/electron-forge.js"),
        plan.value.forgeArgs,
      ],
      [require.resolve("electron-builder/cli.js"), plan.value.builderArgs],
    ] as const) {
      const result = await promisify(execFile)(
        process.execPath,
        [entry, ...args],
        {
          cwd: root,
          maxBuffer: 4 * 1024 * 1024,
          env: { ...process.env, CSC_IDENTITY_AUTO_DISCOVERY: "false" },
        },
      );
      process.stdout.write(result.stdout);
      process.stderr.write(result.stderr);
    }
    const output = join(
      root,
      "out/distribution",
      releaseAssetName(plan.value.target.id, metadata.version),
    );
    if (!(await stat(output)).isFile())
      return {
        ok: false,
        error: {
          kind: "storage",
          message: "The expected native installer was not produced.",
        },
      };
    return { ok: true, value: output };
  } catch (error) {
    if (error && typeof error === "object") {
      if ("stdout" in error && typeof error.stdout === "string")
        process.stdout.write(error.stdout);
      if ("stderr" in error && typeof error.stderr === "string")
        process.stderr.write(error.stderr);
    }
    return {
      ok: false,
      error: {
        kind: "storage",
        message:
          error instanceof Error
            ? error.message
            : "Native installer packaging failed.",
      },
    };
  }
}
const result = await makeDistribution();
if (result.ok) process.stdout.write(`Installer ready: ${result.value}\n`);
else {
  process.stderr.write(`${result.error.message}\n`);
  process.exitCode = 1;
}
