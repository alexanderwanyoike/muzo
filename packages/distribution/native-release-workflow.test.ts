import { readFileSync } from "node:fs";
import { parse as parseToml } from "smol-toml";
import { expect, it } from "vitest";
import { parse } from "yaml";

function workflow(name: string): unknown {
  return parse(
    readFileSync(
      new URL(`../../.github/workflows/${name}`, import.meta.url),
      "utf8",
    ),
  );
}

function nested(root: unknown, ...keys: string[]): unknown {
  return keys.reduce<unknown>((value, key) => {
    if (typeof value !== "object" || value === null) return undefined;
    return Object.getOwnPropertyDescriptor(value, key)?.value as unknown;
  }, root);
}

it("makes the verified Rust installer workflow the sole tagged release publisher", () => {
  const native = workflow("desktop-rust-distribution.yml");
  expect(nested(native, "on", "push", "tags")).toEqual(["v*"]);
  const contextSteps = nested(native, "jobs", "context", "steps");
  expect(Array.isArray(contextSteps)).toBe(true);
  expect(JSON.stringify(contextSteps)).toContain(
    "actions/checkout@d23441a48e516b6c34aea4fa41551a30e30af803",
  );
  expect(JSON.stringify(contextSteps)).toContain("git cat-file -t");
  expect(JSON.stringify(contextSteps)).toContain(
    "git merge-base --is-ancestor HEAD origin/main",
  );
  const release = nested(native, "jobs", "release");
  expect(nested(release, "if")).toBe(
    "github.ref_type == 'tag' && github.event_name == 'push' && github.event.created",
  );
  expect(nested(release, "needs")).toEqual(["context", "assemble"]);
  expect(nested(release, "permissions")).toEqual({ contents: "write" });
  const releaseSteps = JSON.stringify(nested(release, "steps"));
  expect(releaseSteps).toContain("verify-release-tag.sh");
  expect(releaseSteps).toContain("--verify-tag --draft");

  const legacy = workflow("distribution.yml");
  expect(legacy).not.toHaveProperty("on.push");
  expect(legacy).not.toHaveProperty("jobs.release");
});

it("runs the root native installer command for every release target", () => {
  const native = workflow("desktop-rust-distribution.yml");
  const packageJob = nested(native, "jobs", "package");
  const targets = nested(packageJob, "strategy", "matrix", "include");
  expect(targets).toEqual([
    { os: "ubuntu-22.04", label: "linux-x64" },
    { os: "windows-latest", label: "windows-x64" },
    { os: "macos-15", label: "macos-arm64", macho: "arm64" },
    { os: "macos-15-intel", label: "macos-x64", macho: "x86_64" },
  ]);
  const steps = nested(packageJob, "steps");
  const buildStep = Array.isArray(steps)
    ? (steps as unknown[]).find(
        (step) => nested(step, "name") === "Build the native installer",
      )
    : undefined;
  expect(nested(buildStep, "run")).toMatch(
    /yarn desktop:package --target \$\{\{ matrix\.label \}\}/u,
  );
  expect(nested(buildStep, "env", "RUSTUP_TOOLCHAIN")).toBe("1.95");
  expect(JSON.stringify(steps)).not.toContain("hdiutil create");
  expect(JSON.stringify(steps)).not.toContain("--prepackaged");
});

it("starts the packaged AppImage on the Linux upgrade baseline", () => {
  const native = workflow("desktop-rust-distribution.yml");
  const configured = nested(native, "jobs", "package", "steps");
  expect(Array.isArray(configured)).toBe(true);
  const steps = configured as unknown[];
  const build = steps.find(
    (step) => nested(step, "name") === "Build the native installer",
  );
  const smoke = steps.find(
    (step) => nested(step, "name") === "Verify Linux AppImage",
  );
  expect(steps.indexOf(smoke)).toBeGreaterThan(steps.indexOf(build));
  expect(nested(smoke, "if")).toBe("runner.os == 'Linux'");
  expect(nested(smoke, "run")).toMatch(
    /bash tests\/manual\/linux-appimage-start\.sh out\/rust-distribution\/\*\.AppImage/u,
  );
});

it("includes the keyboard library that Iced loads at X11 startup", () => {
  const packager = parseToml(
    readFileSync(
      new URL("../../apps/desktop-rust/Packager.toml", import.meta.url),
      "utf8",
    ),
  );
  expect(nested(packager, "appimage", "libs")).toContain(
    "libxkbcommon-x11.so.0",
  );
});

it("exercises the packaged macOS mini player and retains screenshots for both architectures", () => {
  const native = workflow("desktop-rust-distribution.yml");
  const configured = nested(native, "jobs", "package", "steps");
  expect(Array.isArray(configured)).toBe(true);
  const steps = configured as unknown[];
  const smoke = steps.find((step) => nested(step, "id") === "macos-mini");
  expect(nested(smoke, "if")).toBe("runner.os == 'macOS'");
  expect(nested(smoke, "run")).toMatch(
    /hdiutil attach[\s\S]+swift tests\/manual\/macos-mini\.swift "\$mountpoint\/Muzo\.app" "\$RUNNER_TEMP\/muzo-macos-mini-screenshots"/u,
  );

  const screenshots = steps.find(
    (step) =>
      nested(step, "with", "name") === `muzo-macos-mini-\${{ matrix.label }}`,
  );
  expect(steps.indexOf(screenshots)).toBeGreaterThan(steps.indexOf(smoke));
  expect(nested(screenshots, "uses")).toMatch(/^actions\/upload-artifact@/u);
  expect(nested(screenshots, "if")).toBe(
    `\${{ !cancelled() && runner.os == 'macOS' && steps.macos-mini.outcome != 'skipped' }}`,
  );
  expect(nested(screenshots, "with", "path")).toBe(
    `\${{ runner.temp }}/muzo-macos-mini-screenshots/*.png`,
  );
  expect(nested(smoke, "timeout-minutes")).toBe(3);
});
