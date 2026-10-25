import { createHash } from "node:crypto";
import { readFile } from "node:fs/promises";
import { join } from "node:path";
import { parse } from "yaml";
import { distributionFailure, type DistributionResult } from "./distribution-result";
import { releaseAssetName } from "./release-contract";

function record(value: unknown): value is Record<string, unknown> {
  return Boolean(value) && typeof value === "object" && !Array.isArray(value);
}

export async function inspectUpdateMetadata(directory: string, version: string): Promise<DistributionResult<{ name: string; bytes: Buffer; sha256: string }[]>> {
  try {
    const files = [];
    for (const [target, name] of [["linux-x64", "latest-linux.yml"], ["windows-x64", "latest.yml"]] as const) {
      const bytes = await readFile(join(directory, name));
      const metadata: unknown = parse(bytes.toString("utf8"));
      const installerName = releaseAssetName(target, version);
      const installer = await readFile(join(directory, installerName));
      const sha512 = createHash("sha512").update(installer).digest("base64");
      if (!record(metadata) || metadata.version !== version || metadata.path !== installerName || metadata.sha512 !== sha512 || !Array.isArray(metadata.files) || metadata.files.length !== 1) return distributionFailure("invalidRelease", "Update metadata must match the release installer and version.");
      const entry: unknown = metadata.files[0];
      if (!record(entry) || entry.url !== installerName || entry.sha512 !== sha512 || entry.size !== installer.length) return distributionFailure("invalidRelease", "Update metadata has an invalid URL, size or SHA-512 checksum.");
      files.push({ name, bytes, sha256: createHash("sha256").update(bytes).digest("hex") });
    }
    return { ok: true, value: files };
  } catch {
    return distributionFailure("invalidRelease", "Both Windows and Linux update manifests are required and must be valid YAML.");
  }
}
