import { createHash } from "node:crypto";
import { createReadStream } from "node:fs";
import { lstat, open } from "node:fs/promises";
import { join } from "node:path";
import {
  distributionFailure,
  type DistributionResult,
} from "./distribution-result";
import {
  releaseAssetName,
  type ReleaseArtifact,
  type ReleaseTarget,
} from "./release-contract";

export async function inspectInstaller(
  directory: string,
  target: ReleaseTarget,
  version: string,
): Promise<DistributionResult<ReleaseArtifact>> {
  const name = releaseAssetName(target.id, version);
  try {
    const path = join(directory, name);
    const stats = await lstat(path);
    if (!stats.isFile() || stats.size < 512)
      return distributionFailure(
        "invalidRelease",
        `${name} is not a regular installer file.`,
      );
    const handle = await open(path, "r");
    let valid = false;
    try {
      const header = Buffer.alloc(64);
      await handle.read(header, 0, header.length, 0);
      if (target.platform === "linux") {
        valid =
          header[4] === 2 &&
          header[5] === 1 &&
          header.readUInt16LE(18) === 62 &&
          header.subarray(0, 4).equals(Buffer.from([0x7f, 0x45, 0x4c, 0x46])) &&
          header.subarray(8, 11).equals(Buffer.from([0x41, 0x49, 2]));
      } else if (target.platform === "windows") {
        const offset = header.readUInt32LE(60);
        const signature = Buffer.alloc(4);
        if (offset <= stats.size - 4)
          await handle.read(signature, 0, 4, offset);
        valid =
          header.toString("ascii", 0, 2) === "MZ" &&
          signature.equals(Buffer.from("PE\0\0"));
      } else {
        const trailer = Buffer.alloc(4);
        await handle.read(trailer, 0, 4, stats.size - 512);
        valid = trailer.toString("ascii") === "koly";
      }
    } finally {
      await handle.close();
    }
    if (!valid)
      return distributionFailure(
        "invalidRelease",
        `${name} has an invalid native installer format.`,
      );
    const hash = createHash("sha256");
    for await (const chunk of createReadStream(path)) {
      if (!Buffer.isBuffer(chunk))
        return distributionFailure("storage", `Could not hash ${name}.`);
      hash.update(chunk);
    }
    return {
      ok: true,
      value: {
        platform: target.platform,
        arch: target.arch,
        name,
        size: stats.size,
        sha256: hash.digest("hex"),
      },
    };
  } catch {
    return distributionFailure("storage", `Could not read ${name}.`);
  }
}
