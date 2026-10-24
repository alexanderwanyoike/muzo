import {
  RELEASE_TARGETS,
  type ReleaseTarget,
  type DistributionResult,
} from "@muzo/distribution";

interface DistributionPlan {
  target: ReleaseTarget;
  forgeArgs: string[];
  builderArgs: string[];
}
export function distributionPlan(
  targetId: string,
  hostPlatform: string,
): DistributionResult<DistributionPlan> {
  const target = RELEASE_TARGETS.find(({ id }) => id === targetId);
  const native = {
    linux: { platform: "linux", flag: "--linux" },
    windows: { platform: "win32", flag: "--win" },
    macos: { platform: "darwin", flag: "--mac" },
  };
  if (!target || native[target.platform].platform !== hostPlatform)
    return {
      ok: false,
      error: {
        kind: "invalidRelease",
        message:
          "Choose a supported installer target and build it on its native operating system.",
      },
    };
  const { platform, flag } = native[target.platform];
  const prepackaged = `out/Muzo-${platform}-${target.arch}${platform === "darwin" ? "/Muzo.app" : ""}`;
  return {
    ok: true,
    value: {
      target,
      forgeArgs: ["package", "--arch", target.arch],
      builderArgs: [
        "--prepackaged",
        prepackaged,
        flag,
        `--${target.arch}`,
        "--config",
        "electron-builder.config.json",
        "--publish",
        "never",
      ],
    },
  };
}
