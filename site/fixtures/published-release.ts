import {
  RELEASE_TARGETS,
  releaseAssetName,
  releaseManifestName,
} from "@muzo/distribution";

export default function publishedReleaseFixture() {
  const version = "9.8.7";
  const tag = `v${version}`;
  const artifacts = RELEASE_TARGETS.map(({ id, platform, arch }) => ({
    platform,
    arch,
    name: releaseAssetName(id, version),
    size: 100,
    sha256: "a".repeat(64),
  }));
  const manifest = {
    schemaVersion: 1,
    version,
    tag,
    sourceCommit: "b".repeat(40),
    artifacts,
  };
  const github = {
    tag_name: tag,
    draft: false,
    prerelease: false,
    published_at: "2026-10-24T12:00:00Z",
    assets: [
      ...artifacts,
      { name: releaseManifestName(version), size: 500 },
      { name: "SHA256SUMS", size: 500 },
    ].map(({ name, size }) => ({
      name,
      size,
      browser_download_url: `https://github.com/alexanderwanyoike/muzo/releases/download/${tag}/${name}`,
    })),
  };
  return { version, tag, artifacts, manifest, github };
}
