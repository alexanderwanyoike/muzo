import { readFile } from "node:fs/promises";
import { fileURLToPath } from "node:url";
import type { DistributionResult } from "@muzo/distribution";
import { build } from "vite";
import { validatePublishedDownloads } from "./github-release";
import renderDownloads from "./render-downloads";

async function buildSite(): Promise<DistributionResult<void>> {
  try {
    const [flag, outDir, ...extra] = process.argv.slice(2);
    if ((flag && (flag !== "--outDir" || !outDir)) || extra.length > 0) {
      return {
        ok: false,
        error: {
          kind: "invalidRelease",
          message: "Expected an optional --outDir path.",
        },
      };
    }
    const githubPath = process.env.MUZO_GITHUB_RELEASE_JSON;
    const manifestPath = process.env.MUZO_DISTRIBUTION_MANIFEST;
    if (
      (githubPath ||
        manifestPath ||
        process.env.MUZO_REQUIRE_RELEASE === "1") &&
      (!githubPath || !manifestPath)
    ) {
      return {
        ok: false,
        error: {
          kind: "invalidRelease",
          message:
            "A published release and distribution manifest are required for deployment.",
        },
      };
    }
    let html: string | undefined;
    if (githubPath && manifestPath) {
      const github: unknown = JSON.parse(await readFile(githubPath, "utf8"));
      const manifest: unknown = JSON.parse(
        await readFile(manifestPath, "utf8"),
      );
      const release = validatePublishedDownloads(github, manifest);
      if (!release.ok) return release;
      const rendered = renderDownloads(
        await readFile(new URL("index.html", import.meta.url), "utf8"),
        release.value,
      );
      if (!rendered.ok) return rendered;
      html = rendered.value;
    }
    const renderedHtml = html;
    await build({
      root: fileURLToPath(new URL(".", import.meta.url)),
      base: "/muzo/",
      configFile: false,
      build: { outDir: outDir ?? "dist", emptyOutDir: true },
      plugins: renderedHtml
        ? [
            {
              name: "muzo-release-downloads",
              transformIndexHtml: { order: "pre", handler: () => renderedHtml },
            },
          ]
        : [],
    });
    return { ok: true, value: undefined };
  } catch (error) {
    return {
      ok: false,
      error: {
        kind: "storage",
        message:
          error instanceof Error
            ? error.message
            : "The site could not be built.",
      },
    };
  }
}
const result = await buildSite();
if (!result.ok) {
  process.stderr.write(`${result.error.message}\n`);
  process.exitCode = 1;
}
