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
    bytes[5] = 1;
    bytes.writeUInt16LE(62, 18);
  } else if (target === "windows-x64") {
    bytes.write("MZ");
    bytes.writeUInt32LE(128, 60);
    bytes.write("PE\0\0", 128);
  } else bytes.write("koly", 512);
  return bytes;
}

async function writeUpdateMetadata(directory: string) {
  for (const [target, name] of [
    ["linux-x64", "latest-linux.yml"],
    ["windows-x64", "latest.yml"],
  ] as const) {
    const file = releaseAssetName(target, "0.0.2");
    const sha512 = createHash("sha512")
      .update(installer(target))
      .digest("base64");
    await writeFile(
      join(directory, name),
      `version: 0.0.2\nfiles:\n  - url: ${file}\n    sha512: ${sha512}\n    size: 1024\npath: ${file}\nsha512: ${sha512}\n`,
    );
  }
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
    await writeUpdateMetadata(directory);
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
    for (const name of ["latest-linux.yml", "latest.yml"]) {
      const bytes = await readFile(join(outputDirectory, name));
      expect(sums).toContain(
        `${createHash("sha256").update(bytes).digest("hex")}  ${name}\n`,
      );
    }
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
    await writeUpdateMetadata(directory);
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

it("refuses incomplete or corrupt installers and never overwrites a release set", async () => {
  const { assembleRelease } = await import("./assemble-release");
  const directory = await mkdtemp(join(tmpdir(), "muzo-release-guard-"));
  const input = {
    inputDirectory: directory,
    outputDirectory: join(directory, "release-set"),
    version: "0.0.2",
    tag: "v0.0.2",
    sourceCommit: "a".repeat(40),
  };
  try {
    expect((await assembleRelease(input)).ok).toBe(false);
    await Promise.all(
      RELEASE_TARGETS.map(({ id }) =>
        writeFile(
          join(directory, releaseAssetName(id, "0.0.2")),
          installer(id),
        ),
      ),
    );
    await writeFile(
      join(directory, releaseAssetName("windows-x64", "0.0.2")),
      Buffer.alloc(1024),
    );
    expect((await assembleRelease(input)).ok).toBe(false);
    await writeFile(
      join(directory, releaseAssetName("windows-x64", "0.0.2")),
      installer("windows-x64"),
    );
    await writeUpdateMetadata(directory);
    expect((await assembleRelease(input)).ok).toBe(true);
    const before = await readFile(
      join(input.outputDirectory, "SHA256SUMS"),
      "utf8",
    );
    expect((await assembleRelease(input)).ok).toBe(false);
    expect(
      await readFile(join(input.outputDirectory, "SHA256SUMS"), "utf8"),
    ).toBe(before);
  } finally {
    await rm(directory, { recursive: true, force: true });
  }
});

it("rejects missing, foreign or corrupt update metadata before staging a release", async () => {
  const { assembleRelease } = await import("./assemble-release");
  const directory = await mkdtemp(join(tmpdir(), "muzo-update-metadata-"));
  const input = {
    inputDirectory: directory,
    outputDirectory: join(directory, "out"),
    version: "0.0.2",
    tag: "v0.0.2",
    sourceCommit: "a".repeat(40),
  };
  try {
    await Promise.all(
      RELEASE_TARGETS.map(({ id }) =>
        writeFile(
          join(directory, releaseAssetName(id, "0.0.2")),
          installer(id),
        ),
      ),
    );
    expect((await assembleRelease(input)).ok).toBe(false);
    for (const [from, to] of [
      ["version: 0.0.2", "version: 0.0.3"],
      ["size: 1024", "size: 1025"],
      ["url: Muzo-", "url: https://evil.test/Muzo-"],
      ["sha512: ", "sha512: corrupted"],
    ]) {
      await writeUpdateMetadata(directory);
      const path = join(directory, "latest-linux.yml");
      await writeFile(path, (await readFile(path, "utf8")).replace(from, to));
      expect((await assembleRelease(input)).ok).toBe(false);
    }
    await expect(
      readFile(join(input.outputDirectory, "SHA256SUMS")),
    ).rejects.toMatchObject({ code: "ENOENT" });
  } finally {
    await rm(directory, { recursive: true, force: true });
  }
});
