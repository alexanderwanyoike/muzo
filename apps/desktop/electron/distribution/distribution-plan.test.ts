import { expect, it } from "vitest";
import { distributionPlan } from "./distribution-plan";

it("packages the native architecture and emits an installer without publishing", () => {
  for (const [target, host, arch, path, format] of [
    ["linux-x64", "linux", "x64", "out/Muzo-linux-x64", "--linux"],
    ["windows-x64", "win32", "x64", "out/Muzo-win32-x64", "--win"],
    [
      "macos-arm64",
      "darwin",
      "arm64",
      "out/Muzo-darwin-arm64/Muzo.app",
      "--mac",
    ],
    ["macos-x64", "darwin", "x64", "out/Muzo-darwin-x64/Muzo.app", "--mac"],
  ] as const) {
    const result = distributionPlan(target, host);
    expect(result.ok).toBe(true);
    if (!result.ok) throw new Error(result.error.message);
    expect(result.value.forgeArgs).toEqual(["package", "--arch", arch]);
    expect(result.value.builderArgs).toEqual([
      "--prepackaged",
      path,
      format,
      `--${arch}`,
      "--config",
      "electron-builder.config.json",
      "--publish",
      "never",
    ]);
  }
  expect(distributionPlan("linux-x64", "win32").ok).toBe(false);
  expect(distributionPlan("macos-arm64", "linux").ok).toBe(false);
  expect(distributionPlan("linux-arm64", "linux").ok).toBe(false);
});
