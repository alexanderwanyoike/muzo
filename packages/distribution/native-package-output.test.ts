import { mkdtemp, mkdir, readFile, rm, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { expect, it } from "vitest";
import { publishNativePackage } from "./native-package-output";

it("preserves the existing installer when the update metadata cannot be promoted", async () => {
  const output = await mkdtemp(join(tmpdir(), "muzo-publish-test-"));
  try {
    const installer = "Muzo-0.0.6-linux-x64.AppImage";
    const metadata = "latest-linux.yml";
    const stage = join(output, "stage");
    await mkdir(stage);
    await writeFile(join(output, installer), "old installer");
    await mkdir(join(output, metadata));
    await writeFile(join(stage, installer), "new installer");
    await writeFile(join(stage, metadata), "new metadata");

    await expect(
      publishNativePackage(stage, output, installer, metadata),
    ).rejects.toThrow();
    expect(await readFile(join(output, installer), "utf8")).toBe(
      "old installer",
    );
  } finally {
    await rm(output, { recursive: true, force: true });
  }
});

it("restores the previous installer and metadata when the new metadata cannot move", async () => {
  const output = await mkdtemp(join(tmpdir(), "muzo-publish-rollback-"));
  try {
    const installer = "Muzo-0.0.6-linux-x64.AppImage";
    const metadata = "latest-linux.yml";
    const stage = join(output, "stage");
    await mkdir(stage);
    await writeFile(join(output, installer), "old installer");
    await writeFile(join(output, metadata), "old metadata");
    await writeFile(join(stage, installer), "new installer");

    await expect(
      publishNativePackage(stage, output, installer, metadata),
    ).rejects.toThrow();
    expect(await readFile(join(output, installer), "utf8")).toBe(
      "old installer",
    );
    expect(await readFile(join(output, metadata), "utf8")).toBe("old metadata");
  } finally {
    await rm(output, { recursive: true, force: true });
  }
});

it("identifies recovery files and a way forward after an interrupted promotion", async () => {
  const output = await mkdtemp(join(tmpdir(), "muzo-publish-interrupted-"));
  try {
    const lock = join(output, ".muzo-package-lock");
    const installer = "Muzo-0.0.6-linux-x64.AppImage";
    await mkdir(lock);
    await writeFile(join(lock, installer), "old installer");

    await expect(
      publishNativePackage(join(output, "stage"), output, installer),
    ).rejects.toThrow(/choose another --output/);
    expect(await readFile(join(lock, installer), "utf8")).toBe("old installer");
  } finally {
    await rm(output, { recursive: true, force: true });
  }
});
