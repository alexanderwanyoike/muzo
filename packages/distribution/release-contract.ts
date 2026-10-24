import {
  distributionFailure,
  type DistributionResult,
} from "./distribution-result";

export type { DistributionResult } from "./distribution-result";
export const RELEASE_TARGETS = [
  {
    id: "linux-x64",
    platform: "linux",
    arch: "x64",
    suffix: "linux-x64.AppImage",
  },
  {
    id: "windows-x64",
    platform: "windows",
    arch: "x64",
    suffix: "windows-x64-setup.exe",
  },
  {
    id: "macos-arm64",
    platform: "macos",
    arch: "arm64",
    suffix: "macos-arm64.dmg",
  },
  { id: "macos-x64", platform: "macos", arch: "x64", suffix: "macos-x64.dmg" },
] as const;
export type ReleaseTarget = (typeof RELEASE_TARGETS)[number];
export type ReleaseTargetId = ReleaseTarget["id"];
export function releaseAssetName(
  target: ReleaseTargetId,
  version: string,
): string {
  return `Muzo-${version}-${RELEASE_TARGETS.find(({ id }) => id === target)?.suffix ?? ""}`;
}
export function releaseManifestName(version: string): string {
  return `muzo-${version}-release.json`;
}

export interface ReleaseArtifact {
  platform: ReleaseTarget["platform"];
  arch: ReleaseTarget["arch"];
  name: string;
  size: number;
  sha256: string;
}
export interface ReleaseManifest {
  schemaVersion: 1;
  tag: string;
  version: string;
  sourceCommit: string;
  artifacts: ReleaseArtifact[];
}
const stableVersion = /^(0|[1-9]\d*)\.(0|[1-9]\d*)\.(0|[1-9]\d*)$/;

function record(input: unknown): input is Record<string, unknown> {
  return typeof input === "object" && input !== null;
}
function releaseArtifact(
  input: unknown,
  version: string,
): input is ReleaseArtifact {
  if (!record(input)) return false;
  const target = RELEASE_TARGETS.find(
    ({ platform, arch }) => platform === input.platform && arch === input.arch,
  );
  return (
    target !== undefined &&
    input.name === releaseAssetName(target.id, version) &&
    typeof input.size === "number" &&
    Number.isSafeInteger(input.size) &&
    input.size > 0 &&
    typeof input.sha256 === "string" &&
    /^[0-9a-f]{64}$/.test(input.sha256)
  );
}

export function validateReleaseManifest(
  input: unknown,
): DistributionResult<ReleaseManifest> {
  if (
    !record(input) ||
    input.schemaVersion !== 1 ||
    typeof input.version !== "string" ||
    !stableVersion.test(input.version) ||
    input.tag !== `v${input.version}` ||
    typeof input.sourceCommit !== "string" ||
    !/^[0-9a-f]{40}$/.test(input.sourceCommit) ||
    !Array.isArray(input.artifacts)
  ) {
    return distributionFailure(
      "invalidRelease",
      "Release metadata must identify a stable version and its exact source commit.",
    );
  }
  const { version } = input;
  const artifacts: unknown[] = input.artifacts;
  if (
    artifacts.length !== RELEASE_TARGETS.length ||
    !artifacts.every((asset): asset is ReleaseArtifact =>
      releaseArtifact(asset, version),
    ) ||
    new Set(artifacts.map(({ name }) => name)).size !== RELEASE_TARGETS.length
  ) {
    return distributionFailure(
      "invalidRelease",
      "The release must contain all four supported installers with valid names, sizes and SHA-256 hashes.",
    );
  }
  return {
    ok: true,
    value: {
      schemaVersion: 1,
      tag: input.tag,
      version,
      sourceCommit: input.sourceCommit,
      artifacts,
    },
  };
}
