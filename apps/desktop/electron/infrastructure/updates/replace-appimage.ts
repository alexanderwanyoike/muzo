import { constants } from "node:fs";
import { chmod, copyFile, rename, rm, stat } from "node:fs/promises";
import { randomUUID } from "node:crypto";
import { dirname, isAbsolute, join } from "node:path";
import { failure, success, type Result } from "../../application/result";

const FILE_PERMISSIONS_MASK = 0o777;

export async function replaceAppImage(current: string, downloaded: string): Promise<Result<void>> {
  if (!isAbsolute(current) || !isAbsolute(downloaded) || current === downloaded)
    return failure("invalidInput", "The AppImage update locations are invalid.");
  const staged = join(dirname(current), `.muzo-update-${randomUUID()}.AppImage`);
  try {
    const original = await stat(current);
    const update = await stat(downloaded);
    if (!original.isFile() || !update.isFile() || update.size === 0)
      return failure("invalidInput", "The verified AppImage update is unavailable.");
    await copyFile(downloaded, staged, constants.COPYFILE_EXCL);
    await chmod(staged, original.mode & FILE_PERMISSIONS_MASK);
    // Same-directory rename keeps the old executable intact until replacement succeeds.
    await rename(staged, current);
    return success(undefined);
  } catch {
    return failure("storage", "Could not replace the AppImage. Keep it in a writable folder and try again.");
  } finally {
    await rm(staged, { force: true }).catch(() => undefined);
  }
}
