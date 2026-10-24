// @vitest-environment node
import { execFile } from "node:child_process";
import { mkdtemp, readFile, rm, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { promisify } from "node:util";
import publishedReleaseFixture from "./fixtures/published-release";
import { JSDOM } from "jsdom";
import { expect, it } from "vitest";

it("builds native download links only for a complete published release matching its manifest", async () => {
  const directory = await mkdtemp(join(tmpdir(), "muzo-download-build-"));
  try {
    const { tag, artifacts, manifest, github } = publishedReleaseFixture();
    const releasePath = join(directory, "github.json");
    const manifestPath = join(directory, "manifest.json");
    await writeFile(releasePath, JSON.stringify(github));
    await writeFile(manifestPath, JSON.stringify(manifest));
    await promisify(execFile)(
      "yarn",
      ["site:build", "--outDir", join(directory, "dist")],
      {
        cwd: dirname(dirname(fileURLToPath(import.meta.url))),
        timeout: 15_000,
        env: {
          ...process.env,
          MUZO_GITHUB_RELEASE_JSON: releasePath,
          MUZO_DISTRIBUTION_MANIFEST: manifestPath,
          MUZO_REQUIRE_RELEASE: "1",
        },
      },
    );
    const dom = new JSDOM(
      await readFile(join(directory, "dist/index.html"), "utf8"),
    );
    try {
      const links = [
        ...dom.window.document.querySelectorAll<HTMLAnchorElement>(
          "[data-release-download] a",
        ),
      ];
      expect(links).toHaveLength(4);
      expect(links.map((link) => link.href).sort()).toEqual(
        artifacts
          .map(
            ({ name }) =>
              `https://github.com/alexanderwanyoike/muzo/releases/download/${tag}/${name}`,
          )
          .sort(),
      );
      expect(
        dom.window.document.querySelector("[data-release-status]")?.textContent,
      ).toContain(tag);
      expect(
        dom.window.document
          .querySelector("[data-release-checksums] a")
          ?.getAttribute("href"),
      ).toBe(
        `https://github.com/alexanderwanyoike/muzo/releases/download/${tag}/SHA256SUMS`,
      );
    } finally {
      dom.window.close();
    }
  } finally {
    await rm(directory, { recursive: true, force: true });
  }
}, 20_000);
