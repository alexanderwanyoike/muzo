import { expect, it } from "vitest";
import publishedReleaseFixture from "./fixtures/published-release";
import { validatePublishedDownloads } from "./github-release";

it("rejects ambiguous asset names rather than selecting an unverified download URL", () => {
  const { github, manifest } = publishedReleaseFixture();
  const first = github.assets[0];
  const result = validatePublishedDownloads(
    {
      ...github,
      assets: [
        {
          ...first,
          browser_download_url: "https://example.invalid/unverified.exe",
        },
        ...github.assets,
      ],
    },
    manifest,
  );
  expect(result).toMatchObject({
    ok: false,
    error: { kind: "invalidRelease" },
  });
});

it.each(["draft", "prerelease"] as const)("rejects a %s release", (flag) => {
  const { github, manifest } = publishedReleaseFixture();
  expect(
    validatePublishedDownloads({ ...github, [flag]: true }, manifest).ok,
  ).toBe(false);
});

it("rejects incomplete installer sets, missing checksums and mismatched tags", () => {
  const { github, manifest } = publishedReleaseFixture();
  for (const omitted of github.assets) {
    expect(
      validatePublishedDownloads(
        {
          ...github,
          assets: github.assets.filter(({ name }) => name !== omitted.name),
        },
        manifest,
      ).ok,
    ).toBe(false);
  }
  expect(
    validatePublishedDownloads({ ...github, tag_name: "v1.0.0" }, manifest).ok,
  ).toBe(false);
  expect(
    validatePublishedDownloads({ ...github, published_at: null }, manifest).ok,
  ).toBe(false);
});

it("rejects installer URLs, sizes and available SHA-256 digests that differ from the manifest", () => {
  const { github, manifest } = publishedReleaseFixture();
  for (const changes of [
    { size: 101 },
    { digest: `sha256:${"c".repeat(64)}` },
    { browser_download_url: "https://example.invalid/installer" },
  ]) {
    const assets = github.assets.map((entry, index) =>
      index === 0 ? { ...entry, ...changes } : entry,
    );
    expect(validatePublishedDownloads({ ...github, assets }, manifest).ok).toBe(
      false,
    );
  }
});
