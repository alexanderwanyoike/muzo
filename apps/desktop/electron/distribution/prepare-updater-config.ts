import { mkdir, writeFile } from "node:fs/promises";
import { join } from "node:path";
import { MUZO_UPDATE_FEED } from "../../shared/update-feed";
import { failure, success, type Result } from "../application/result";

export async function prepareUpdaterConfig(
  prepackagedDirectory: string,
): Promise<Result<void>> {
  try {
    const resources = join(prepackagedDirectory, "resources");
    await mkdir(resources, { recursive: true });
    // electron-builder skips its afterPack config writer when using Forge's prepackaged payload.
    const config = { ...MUZO_UPDATE_FEED, updaterCacheDirName: "muzo-updater" };
    await writeFile(
      join(resources, "app-update.yml"),
      `${Object.entries(config)
        .map(([key, value]) => `${key}: ${value}`)
        .join("\n")}\n`,
    );
    return success(undefined);
  } catch {
    return failure(
      "storage",
      "Could not prepare the installed updater configuration.",
    );
  }
}
