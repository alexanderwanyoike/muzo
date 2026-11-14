import { readFileSync } from "node:fs";
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
  expect(JSON.stringify(contextSteps)).toContain("git cat-file -t");
  expect(JSON.stringify(contextSteps)).toContain(
    "git merge-base --is-ancestor HEAD origin/main",
  );
  const release = nested(native, "jobs", "release");
  expect(nested(release, "if")).toBe(
    "github.ref_type == 'tag' && github.event_name == 'push'",
  );
  expect(nested(release, "needs")).toEqual(["context", "assemble"]);
  expect(nested(release, "permissions")).toEqual({ contents: "write" });
  expect(JSON.stringify(nested(release, "steps"))).toContain(
    "--verify-tag --draft",
  );

  const legacy = workflow("distribution.yml");
  expect(legacy).not.toHaveProperty("on.push");
  expect(legacy).not.toHaveProperty("jobs.release");
});
