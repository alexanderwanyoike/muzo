import { lstat, mkdir, rename, rmdir } from "node:fs/promises";
import { join } from "node:path";

async function existingFile(path: string): Promise<boolean> {
  try {
    const info = await lstat(path);
    if (!info.isFile())
      throw new Error(`Installer output path is not a file: ${path}`);
    return true;
  } catch (error) {
    if (
      error &&
      typeof error === "object" &&
      "code" in error &&
      error.code === "ENOENT"
    )
      return false;
    throw error;
  }
}

export async function publishNativePackage(
  stage: string,
  output: string,
  installer: string,
  metadata?: string,
): Promise<void> {
  const lock = join(output, ".muzo-package-lock");
  await mkdir(lock);
  const installerPath = join(output, installer);
  const metadataPath = metadata ? join(output, metadata) : undefined;
  const oldInstaller = join(lock, installer);
  const oldMetadata = metadata ? join(lock, metadata) : undefined;
  let previousMetadataMoved = false;
  let previousInstallerMoved = false;
  let installerPromoted = false;
  try {
    const hasInstaller = await existingFile(installerPath);
    const hasMetadata = metadataPath && (await existingFile(metadataPath));
    if (hasMetadata && metadataPath && oldMetadata) {
      await rename(metadataPath, oldMetadata);
      previousMetadataMoved = true;
    }
    if (hasInstaller) {
      await rename(installerPath, oldInstaller);
      previousInstallerMoved = true;
    }
    await rename(join(stage, installer), installerPath);
    installerPromoted = true;
    if (metadata && metadataPath)
      await rename(join(stage, metadata), metadataPath);
  } catch (error) {
    if (installerPromoted) await rename(installerPath, join(stage, installer));
    if (previousInstallerMoved) await rename(oldInstaller, installerPath);
    if (previousMetadataMoved && metadataPath && oldMetadata)
      await rename(oldMetadata, metadataPath);
    await rmdir(lock);
    throw error;
  }
  if (previousInstallerMoved)
    await rename(oldInstaller, join(stage, ".previous-installer"));
  if (previousMetadataMoved && oldMetadata)
    await rename(oldMetadata, join(stage, ".previous-metadata"));
  await rmdir(lock);
}
