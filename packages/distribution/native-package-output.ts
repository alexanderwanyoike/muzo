import {
  copyFile,
  link,
  lstat,
  mkdir,
  rename,
  rmdir,
  unlink,
} from "node:fs/promises";
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

async function backUp(source: string, backup: string): Promise<void> {
  try {
    await link(source, backup);
  } catch {
    await copyFile(source, backup);
  }
}

export async function publishNativePackage(
  stage: string,
  output: string,
  installer: string,
  metadata?: string,
): Promise<void> {
  const lock = join(output, ".muzo-package-lock");
  try {
    await mkdir(lock);
  } catch (error) {
    if (
      error &&
      typeof error === "object" &&
      "code" in error &&
      error.code === "EEXIST"
    )
      throw new Error(
        `Installer output is busy or an earlier build was interrupted. Check ${lock} for recovery files, or choose another --output.`,
        { cause: error },
      );
    throw error;
  }
  const installerPath = join(output, installer);
  const metadataPath = metadata ? join(output, metadata) : undefined;
  const oldInstaller = join(lock, installer);
  const oldMetadata = metadata ? join(lock, metadata) : undefined;
  let metadataBackedUp = false;
  let installerBackedUp = false;
  let metadataRemoved = false;
  let installerPromoted = false;
  try {
    const hasInstaller = await existingFile(installerPath);
    const hasMetadata = metadataPath && (await existingFile(metadataPath));
    if (hasMetadata && metadataPath && oldMetadata) {
      await backUp(metadataPath, oldMetadata);
      metadataBackedUp = true;
    }
    if (hasInstaller) {
      await backUp(installerPath, oldInstaller);
      installerBackedUp = true;
    }
    if (hasMetadata && metadataPath) {
      await unlink(metadataPath);
      metadataRemoved = true;
    }
    await rename(join(stage, installer), installerPath);
    installerPromoted = true;
    if (metadata && metadataPath)
      await rename(join(stage, metadata), metadataPath);
  } catch (error) {
    try {
      if (installerPromoted) {
        if (installerBackedUp) await rename(oldInstaller, installerPath);
        else await unlink(installerPath);
      } else if (installerBackedUp) await unlink(oldInstaller);
      if (metadataBackedUp && metadataPath && oldMetadata) {
        if (metadataRemoved) await rename(oldMetadata, metadataPath);
        else await unlink(oldMetadata);
      }
    } catch (rollbackError) {
      throw new Error(
        `Could not restore the installer output. Check ${lock} for recovery files, or choose another --output.`,
        { cause: new AggregateError([error, rollbackError]) },
      );
    }
    await rmdir(lock);
    throw error;
  }
  if (installerBackedUp) await unlink(oldInstaller);
  if (metadataBackedUp && oldMetadata) await unlink(oldMetadata);
  await rmdir(lock);
}
