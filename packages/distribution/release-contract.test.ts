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
