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

const nativeHeaderBytes = 64;
const dmgTrailerBytes = 512;
const elfClassOffset = 4;
const elf64BitClass = 2;
const elfEncodingOffset = 5;
const elfLittleEndian = 1;
const elfMachineOffset = 18;
const elfX64Machine = 62;
const appImageMagicOffset = 8;
const peHeaderPointerOffset = 60;
const elfMagic = Buffer.from([0x7f, 0x45, 0x4c, 0x46]);
const appImageType2Magic = Buffer.from([0x41, 0x49, 2]);
const peMagic = Buffer.from("PE\0\0");
const dmgMagic = Buffer.from("koly");

export async function inspectInstaller(
  directory: string,
  target: ReleaseTarget,
  version: string,
): Promise<DistributionResult<ReleaseArtifact>> {
  const name = releaseAssetName(target.id, version);
  try {
    const path = join(directory, name);
    const stats = await lstat(path);
    if (!stats.isFile() || stats.size < dmgTrailerBytes)
      return distributionFailure(
        "invalidRelease",
        `${name} is not a regular installer file.`,
      );
    const handle = await open(path, "r");
    let valid = false;
    try {
      const header = Buffer.alloc(nativeHeaderBytes);
      await handle.read(header, 0, header.length, 0);
      if (target.platform === "linux") {
        valid =
          header[elfClassOffset] === elf64BitClass &&
          header[elfEncodingOffset] === elfLittleEndian &&
          header.readUInt16LE(elfMachineOffset) === elfX64Machine &&
          header.subarray(0, elfMagic.length).equals(elfMagic) &&
          header
            .subarray(
              appImageMagicOffset,
              appImageMagicOffset + appImageType2Magic.length,
            )
            .equals(appImageType2Magic);
      } else if (target.platform === "windows") {
        const offset = header.readUInt32LE(peHeaderPointerOffset);
        const signature = Buffer.alloc(peMagic.length);
        if (offset <= stats.size - peMagic.length)
          await handle.read(signature, 0, signature.length, offset);
        valid =
          header.toString("ascii", 0, 2) === "MZ" && signature.equals(peMagic);
      } else {
        const trailer = Buffer.alloc(dmgMagic.length);
        await handle.read(
          trailer,
          0,
          trailer.length,
          stats.size - dmgTrailerBytes,
        );
        valid = trailer.equals(dmgMagic);
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
