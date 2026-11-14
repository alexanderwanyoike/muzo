import { expect, it } from "vitest";
import { releaseContext } from "./release-context";

it("binds a release candidate to matching package versions and its exact source commit", () => {
  const context = {
    rootVersion: "0.0.2",
    desktopVersion: "0.0.2",
    sourceCommit: "a".repeat(40),
  };
  expect(releaseContext(context)).toEqual({
    ok: true,
    value: { version: "0.0.2", tag: "v0.0.2", sourceCommit: "a".repeat(40) },
  });
  expect(releaseContext({ ...context, tag: "v0.0.2" }).ok).toBe(true);
  for (const input of [
    { ...context, rootVersion: "0.0.1" },
    { ...context, tag: "v0.0.1" },
    { ...context, rootVersion: "0.0.2-beta.1", desktopVersion: "0.0.2-beta.1" },
    { ...context, sourceCommit: "main" },
    { ...context, rootVersion: "../0.0.2", desktopVersion: "../0.0.2" },
  ])
    expect(releaseContext(input).ok).toBe(false);
});

it("rejects a candidate when any native desktop version disagrees", () => {
  const context = {
    rootVersion: "0.0.6",
    desktopVersion: "0.0.6",
    sourceCommit: "a".repeat(40),
    native: { workspace: "0.0.6", binary: "0.0.6", packager: "0.0.6" },
  };

  expect(releaseContext(context).ok).toBe(true);
  for (const part of ["workspace", "binary", "packager"] as const) {
    expect(
      releaseContext({
        ...context,
        native: { ...context.native, [part]: "0.0.5" },
      }).ok,
    ).toBe(false);
  }
});
