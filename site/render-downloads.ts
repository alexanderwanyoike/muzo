import type { DistributionResult, ReleaseTargetId } from "@muzo/distribution";
import { JSDOM } from "jsdom";
import type { PublishedDownloads } from "./github-release";

const labels: Record<ReleaseTargetId, string> = {
  "linux-x64": "Download AppImage",
  "windows-x64": "Download installer",
  "macos-arm64": "Download for Apple silicon",
  "macos-x64": "Download for Intel",
};
export default function renderDownloads(
  html: string,
  release: PublishedDownloads,
): DistributionResult<string> {
  const dom = new JSDOM(html);
  try {
    const { document } = dom.window;
    const status = document.querySelector("[data-release-status]");
    const checksums = document.querySelector("[data-release-checksums]");
    if (
      !status ||
      !checksums ||
      release.downloads.some(
        ({ target }) =>
          !document.querySelector(`[data-release-download="${target}"]`),
      )
    ) {
      return {
        ok: false,
        error: {
          kind: "invalidRelease",
          message: "The site is missing its release download sections.",
        },
      };
    }
    for (const download of release.downloads) {
      const container = document.querySelector(
        `[data-release-download="${download.target}"]`,
      );
      const link = document.createElement("a");
      link.className =
        download.target === "macos-x64"
          ? "download-button download-button--secondary"
          : "download-button";
      link.href = download.url;
      link.textContent = labels[download.target];
      link.setAttribute(
        "aria-label",
        `${labels[download.target]} · Muzo ${release.manifest.tag}`,
      );
      container?.replaceChildren(link);
    }
    const date = new Intl.DateTimeFormat("en-GB", {
      day: "numeric",
      month: "long",
      year: "numeric",
      timeZone: "UTC",
    }).format(new Date(release.publishedAt));
    status.textContent = `${release.manifest.tag} · ${date}. Choose the package for your desktop.`;
    const version = document.querySelector("[data-release-version]");
    if (version) version.textContent = release.manifest.tag;
    const checksumLink = document.createElement("a");
    checksumLink.href = release.checksumsUrl;
    checksumLink.textContent = "SHA-256 checksums";
    checksums.replaceChildren(checksumLink);
    return { ok: true, value: dom.serialize() };
  } finally {
    dom.window.close();
  }
}
