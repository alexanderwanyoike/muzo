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
  desktopVersion: unknown;
  native?: { workspace: unknown; binary: unknown; packager: unknown };
  sourceCommit: unknown;
  tag?: unknown;
}
export interface ReleaseContext {
  version: string;
  tag: string;
  sourceCommit: string;
}
export function releaseContext(
  input: ReleaseContextInput,
): DistributionResult<ReleaseContext> {
  const { rootVersion, desktopVersion, native, sourceCommit, tag } = input;
  if (
    native !== undefined &&
    [native.workspace, native.binary, native.packager].some(
      (version) => version !== rootVersion,
    )
  )
    return distributionFailure(
      "invalidRelease",
      "The native workspace, Cargo and Packager versions must match the root version.",
    );
  if (
    !isStableReleaseVersion(rootVersion) ||
    rootVersion !== desktopVersion ||
    !isReleaseSourceCommit(sourceCommit) ||
    (tag !== undefined && tag !== `v${rootVersion}`)
  )
    return distributionFailure(
      "invalidRelease",
      "Root and desktop versions must match a stable release tag and exact source commit.",
    );
  return {
    ok: true,
    value: { version: rootVersion, tag: `v${rootVersion}`, sourceCommit },
  };
}
