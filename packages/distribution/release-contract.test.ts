import { expect, it } from "vitest";
import {
  RELEASE_TARGETS,
  releaseAssetName,
  releaseManifestName,
} from "./release-contract";

it("gives each supported installer a stable versioned name", () => {
  expect(
    RELEASE_TARGETS.map(({ id }) => releaseAssetName(id, "0.0.2")),
  ).toEqual([
    "Muzo-0.0.2-linux-x64.AppImage",
    "Muzo-0.0.2-windows-x64-setup.exe",
    "Muzo-0.0.2-macos-arm64.dmg",
    "Muzo-0.0.2-macos-x64.dmg",
  ]);
  expect(releaseManifestName("0.0.2")).toBe("muzo-0.0.2-release.json");
});

it("accepts a complete release manifest and rejects missing or mismatched installers", async () => {
  const { validateReleaseManifest } = await import("./release-contract");
  const manifest = {
    schemaVersion: 1,
    tag: "v0.0.2",
    version: "0.0.2",
    sourceCommit: "a".repeat(40),
    artifacts: RELEASE_TARGETS.map((target) => ({
      platform: target.platform,
      arch: target.arch,
      name: releaseAssetName(target.id, "0.0.2"),
      size: 123,
      sha256: "b".repeat(64),
    })),
  };
  expect(validateReleaseManifest(manifest)).toEqual({
    ok: true,
    value: manifest,
  });
  for (const invalid of [
    null,
    { ...manifest, schemaVersion: 2 },
    { ...manifest, tag: "v0.0.3" },
    { ...manifest, tag: "v0.0.2-beta.1" },
    { ...manifest, sourceCommit: "main" },
    { ...manifest, artifacts: manifest.artifacts.slice(1) },
    {
      ...manifest,
      artifacts: manifest.artifacts.map((asset, index) =>
        index === 0 ? { ...asset, sha256: "not-a-hash" } : asset,
      ),
    },
    {
      ...manifest,
      artifacts: manifest.artifacts.map((asset, index) =>
        index === 0 ? { ...asset, name: "other.exe" } : asset,
      ),
    },
    {
      ...manifest,
      artifacts: manifest.artifacts.map((asset, index) =>
        index === 0 ? { ...asset, size: 0 } : asset,
      ),
    },
    {
      ...manifest,
      artifacts: [manifest.artifacts[0], ...manifest.artifacts.slice(0, 3)],
    },
  ]) {
    expect(validateReleaseManifest(invalid).ok).toBe(false);
  }
});
