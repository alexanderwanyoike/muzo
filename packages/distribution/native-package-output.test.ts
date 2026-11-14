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
