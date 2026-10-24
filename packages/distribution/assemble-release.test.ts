import { createHash } from "node:crypto";
import { mkdtemp, readFile, writeFile, rm } from "node:fs/promises";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { expect, it } from "vitest";
import {
  RELEASE_TARGETS,
  releaseAssetName,
  releaseManifestName,
  validateReleaseManifest,
} from "./release-contract";

function installer(target: string): Buffer {
  const bytes = Buffer.alloc(1024);
  if (target === "linux-x64") {
    bytes.set([0x7f, 0x45, 0x4c, 0x46]);
    bytes.set([0x41, 0x49, 2], 8);
    bytes[4] = 2;
    bytes.writeUInt16LE(62, 18);
  } else if (target === "windows-x64") {
    bytes.write("MZ");
    bytes.writeUInt32LE(128, 60);
    bytes.write("PE\0\0", 128);
  } else bytes.write("koly", 512);
  return bytes;
}

it("assembles a complete verified installer set with matching manifest and checksums", async () => {
  const { assembleRelease } = await import("./assemble-release");
  const directory = await mkdtemp(join(tmpdir(), "muzo-release-"));
  try {
    await Promise.all(
      RELEASE_TARGETS.map(({ id }) =>
        writeFile(
          join(directory, releaseAssetName(id, "0.0.2")),
          installer(id),
        ),
      ),
    );
    const outputDirectory = join(directory, "release-set");
    const result = await assembleRelease({
      inputDirectory: directory,
      outputDirectory,
      version: "0.0.2",
      tag: "v0.0.2",
      sourceCommit: "a".repeat(40),
    });
    expect(result.ok).toBe(true);
    const manifestBytes = await readFile(
      join(outputDirectory, releaseManifestName("0.0.2")),
    );
    const manifest = validateReleaseManifest(
      JSON.parse(manifestBytes.toString()),
    );
    expect(manifest.ok).toBe(true);
    if (!manifest.ok) throw new Error(manifest.error.message);
    const sums = await readFile(join(outputDirectory, "SHA256SUMS"), "utf8");
    await Promise.all(
      manifest.value.artifacts.map(async (artifact) => {
        const bytes = await readFile(join(outputDirectory, artifact.name));
        expect(artifact.size).toBe(bytes.length);
        expect(artifact.sha256).toBe(
          createHash("sha256").update(bytes).digest("hex"),
        );
        expect(sums).toContain(`${artifact.sha256}  ${artifact.name}\n`);
      }),
    );
    expect(sums).toContain(
      `${createHash("sha256").update(manifestBytes).digest("hex")}  ${releaseManifestName("0.0.2")}\n`,
    );
  } finally {
    await rm(directory, { recursive: true, force: true });
  }
});

it("rejects a renamed AppImage for another CPU before creating release output", async () => {
  const { assembleRelease } = await import("./assemble-release");
  const directory = await mkdtemp(join(tmpdir(), "muzo-release-invalid-"));
  try {
    await Promise.all(
      RELEASE_TARGETS.map(({ id }) =>
        writeFile(
          join(directory, releaseAssetName(id, "0.0.2")),
          installer(id),
        ),
      ),
    );
    const wrongCpu = installer("linux-x64");
    wrongCpu.writeUInt16LE(183, 18);
    await writeFile(
      join(directory, releaseAssetName("linux-x64", "0.0.2")),
      wrongCpu,
    );
    const outputDirectory = join(directory, "release-set");
    expect(
      (
        await assembleRelease({
          inputDirectory: directory,
          outputDirectory,
          version: "0.0.2",
          tag: "v0.0.2",
          sourceCommit: "a".repeat(40),
        })
      ).ok,
    ).toBe(false);
    await expect(
      readFile(join(outputDirectory, "SHA256SUMS")),
    ).rejects.toMatchObject({ code: "ENOENT" });
  } finally {
    await rm(directory, { recursive: true, force: true });
  }
});
