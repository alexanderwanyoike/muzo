import {
  distributionFailure,
  type DistributionResult,
} from "./distribution-result";
import {
  isStableReleaseVersion,
  isReleaseSourceCommit,
} from "./release-contract";

interface ReleaseContextInput {
  rootVersion: unknown;
  native: { workspace: unknown; binary: unknown; packager: unknown };
  sourceCommit: unknown;
  tag?: unknown;
}
export interface ReleaseContext {
  version: string;
  tag: string;
  sourceCommit: string;
}
export function releaseVersion({
  rootVersion,
  native,
}: Omit<
  ReleaseContextInput,
  "sourceCommit" | "tag"
>): DistributionResult<string> {
  if (!isStableReleaseVersion(rootVersion))
    return distributionFailure(
      "invalidRelease",
      "Root version must be a stable release version.",
    );
  if (
    [native.workspace, native.binary, native.packager].some(
      (version) => version !== rootVersion,
    )
  )
    return distributionFailure(
      "invalidRelease",
      "The native workspace, Cargo and Packager versions must match the root version.",
    );
  return { ok: true, value: rootVersion };
}

export function releaseContext(
  input: ReleaseContextInput,
): DistributionResult<ReleaseContext> {
  const version = releaseVersion(input);
  if (!version.ok) return version;
  const { sourceCommit, tag } = input;
  if (
    !isReleaseSourceCommit(sourceCommit) ||
    (tag !== undefined && tag !== `v${version.value}`)
  )
    return distributionFailure(
      "invalidRelease",
      "The release requires a matching tag and the exact source commit.",
    );
  return {
    ok: true,
    value: { version: version.value, tag: `v${version.value}`, sourceCommit },
  };
}
