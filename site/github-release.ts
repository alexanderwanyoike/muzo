import {
  RELEASE_TARGETS,
  releaseAssetName,
  releaseManifestName,
  validateReleaseManifest,
  type DistributionResult,
  type ReleaseManifest,
  type ReleaseTargetId,
} from "@muzo/distribution";

interface PublishedAsset {
  name: string;
  size: number;
  browser_download_url: string;
  digest?: string | null;
}
export interface PublishedDownloads {
  manifest: ReleaseManifest;
  publishedAt: string;
  downloads: { target: ReleaseTargetId; url: string; size: number }[];
  checksumsUrl: string;
}
const repository = "https://github.com/alexanderwanyoike/muzo";
function record(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null;
}
function asset(value: unknown): value is PublishedAsset {
  return (
    record(value) &&
    typeof value.name === "string" &&
    typeof value.size === "number" &&
    Number.isSafeInteger(value.size) &&
    value.size > 0 &&
    typeof value.browser_download_url === "string" &&
    (value.digest === undefined ||
      value.digest === null ||
      typeof value.digest === "string")
  );
}
function invalid(message: string): DistributionResult<never> {
  return { ok: false, error: { kind: "invalidRelease", message } };
}
export function validatePublishedDownloads(
  github: unknown,
  manifestInput: unknown,
): DistributionResult<PublishedDownloads> {
  const manifest = validateReleaseManifest(manifestInput);
  if (!manifest.ok) return manifest;
  if (
    !record(github) ||
    github.tag_name !== manifest.value.tag ||
    github.draft !== false ||
    github.prerelease !== false ||
    typeof github.published_at !== "string" ||
    !Number.isFinite(Date.parse(github.published_at)) ||
    !Array.isArray(github.assets)
  ) {
    return invalid(
      "Downloads require a published stable GitHub release matching the distribution manifest.",
    );
  }
  const values: unknown[] = github.assets;
  if (
    !values.every(asset) ||
    new Set(values.map((entry) => entry.name)).size !== values.length
  )
    return invalid("Published release asset metadata is invalid.");
  const prefix = `${repository}/releases/download/${manifest.value.tag}/`;
  const required = [
    ...manifest.value.artifacts.map(({ name }) => name),
    releaseManifestName(manifest.value.version),
    "SHA256SUMS",
  ];
  if (
    !required.every(
      (name) =>
        values.filter(
          (item) =>
            item.name === name &&
            item.browser_download_url === `${prefix}${name}`,
        ).length === 1,
    )
  ) {
    return invalid(
      "The published release is missing a supported installer, its manifest or SHA256SUMS.",
    );
  }
  const downloads: PublishedDownloads["downloads"] = [];
  for (const target of RELEASE_TARGETS) {
    const name = releaseAssetName(target.id, manifest.value.version);
    const entry = values.find((item) => item.name === name);
    const expected = manifest.value.artifacts.find(
      (item) => item.name === name,
    );
    if (
      !entry ||
      !expected ||
      entry.size !== expected.size ||
      (entry.digest && entry.digest !== `sha256:${expected.sha256}`)
    ) {
      return invalid(
        "Published installer sizes or SHA-256 digests do not match the distribution manifest.",
      );
    }
    downloads.push({
      target: target.id,
      url: entry.browser_download_url,
      size: entry.size,
    });
  }
  return {
    ok: true,
    value: {
      manifest: manifest.value,
      publishedAt: github.published_at,
      downloads,
      checksumsUrl: `${prefix}SHA256SUMS`,
    },
  };
}
