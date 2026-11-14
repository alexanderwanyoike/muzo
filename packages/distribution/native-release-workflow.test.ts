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

it("makes the verified Rust installer workflow the sole tagged release publisher", () => {
  expect(workflow("desktop-rust-distribution.yml")).toMatchObject({
    on: { push: { tags: ["v*"] } },
    jobs: {
      context: {
        steps: expect.arrayContaining([
          expect.objectContaining({
            if: "github.ref_type == 'tag'",
            run: expect.stringContaining(
              "git merge-base --is-ancestor HEAD origin/main",
            ),
          }),
        ]),
      },
      release: {
        if: "github.ref_type == 'tag'",
        needs: ["context", "assemble"],
        permissions: { contents: "write" },
        steps: expect.arrayContaining([
          expect.objectContaining({
            run: expect.stringContaining("--verify-tag --draft"),
          }),
        ]),
      },
    },
  });
  expect(workflow("distribution.yml")).not.toHaveProperty("on.push");
  expect(workflow("distribution.yml")).not.toHaveProperty("jobs.release");
});
