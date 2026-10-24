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
