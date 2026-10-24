import {
  distributionFailure,
  type DistributionResult,
} from "./distribution-result";

interface ReleaseContextInput {
  rootVersion: unknown;
  desktopVersion: unknown;
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
  const { rootVersion, desktopVersion, sourceCommit, tag } = input;
  if (
    typeof rootVersion !== "string" ||
    !/^(0|[1-9]\d*)\.(0|[1-9]\d*)\.(0|[1-9]\d*)$/.test(rootVersion) ||
    rootVersion !== desktopVersion ||
    typeof sourceCommit !== "string" ||
    !/^[0-9a-f]{40}$/.test(sourceCommit) ||
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
