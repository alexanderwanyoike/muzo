import { createHash } from "node:crypto";
import { constants } from "node:fs";
import { copyFile, mkdir, writeFile } from "node:fs/promises";
import { join } from "node:path";
import {
  distributionFailure,
  type DistributionResult,
} from "./distribution-result";
import { inspectInstaller } from "./inspect-installer";
import {
  RELEASE_TARGETS,
  releaseManifestName,
  validateReleaseManifest,
  type ReleaseArtifact,
  type ReleaseManifest,
} from "./release-contract";

export interface ReleaseAssemblyInput {
  inputDirectory: string;
  outputDirectory: string;
  tag: string;
  version: string;
  sourceCommit: string;
}
export async function assembleRelease(
  input: ReleaseAssemblyInput,
): Promise<DistributionResult<ReleaseManifest>> {
  const { tag, version, sourceCommit, inputDirectory, outputDirectory } = input;
  if (
    !/^(0|[1-9]\d*)\.(0|[1-9]\d*)\.(0|[1-9]\d*)$/.test(version) ||
    tag !== `v${version}` ||
    !/^[0-9a-f]{40}$/.test(sourceCommit)
  )
    return distributionFailure(
      "invalidRelease",
      "Release version, tag and source commit are inconsistent.",
    );
  const inspected = await Promise.all(
    RELEASE_TARGETS.map((target) =>
      inspectInstaller(inputDirectory, target, version),
    ),
  );
  const artifacts: ReleaseArtifact[] = [];
  for (const result of inspected) {
    if (!result.ok) return result;
    artifacts.push(result.value);
  }
  const validated = validateReleaseManifest({
    schemaVersion: 1,
    tag,
    version,
    sourceCommit,
    artifacts,
  });
  if (!validated.ok) return validated;
  try {
    await mkdir(outputDirectory);
    await Promise.all(
      artifacts.map(({ name }) =>
        copyFile(
          join(inputDirectory, name),
          join(outputDirectory, name),
          constants.COPYFILE_EXCL,
        ),
      ),
    );
    const manifest = `${JSON.stringify(validated.value, null, 2)}\n`;
    const manifestName = releaseManifestName(version);
    await writeFile(join(outputDirectory, manifestName), manifest, {
      flag: "wx",
    });
    const sums = [
      ...artifacts.map(({ name, sha256 }) => `${sha256}  ${name}`),
      `${createHash("sha256").update(manifest).digest("hex")}  ${manifestName}`,
    ].join("\n");
    await writeFile(join(outputDirectory, "SHA256SUMS"), `${sums}\n`, {
      flag: "wx",
    });
    return validated;
  } catch {
    return distributionFailure(
      "storage",
      "Could not create the release set; existing output directories are never overwritten.",
    );
  }
}
