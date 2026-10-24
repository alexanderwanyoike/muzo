import { createHash, randomUUID } from "node:crypto";
import { mkdir, open, readdir, rename, rm, stat } from "node:fs/promises";
import { join } from "node:path";
import { failure, type Result } from "../../../shared/result";
import type { DropboxAudioDownload } from "../../application/interfaces/dropbox-audio-download";

export class DropboxAudioDownloader implements DropboxAudioDownload {
  constructor(
    private readonly directory: string,
    private readonly request: typeof fetch = fetch,
    private readonly storageLimitBytes = 1024 * 1024 * 1024,
  ) {}

  async download(accessToken: string, trackKey: string, remotePath: string): Promise<Result<string>> {
    const name = createHash("sha256").update(trackKey).digest("hex");
    const target = join(this.directory, name);
    const temporary = `${target}.${randomUUID()}.part`;
    try {
      const response = await this.request("https://content.dropboxapi.com/2/files/download", {
        method: "POST",
        headers: {
          Authorization: `Bearer ${accessToken}`,
          "Dropbox-API-Arg": JSON.stringify({ path: remotePath }).replace(/[\u007f-\uffff]/g,
            (character) => `\\u${character.charCodeAt(0).toString(16).padStart(4, "0")}`),
        },
        signal: AbortSignal.timeout(120_000),
      });
      if (response.status !== 200 || !response.body) {
        await response.body?.cancel();
        return downloadFailure();
      }
      await mkdir(this.directory, { recursive: true, mode: 0o700 });
      const file = await open(temporary, "wx", 0o600);
      const reader = response.body.getReader();
      try {
        let bytes = 0;
        let chunk = await reader.read();
        while (!chunk.done) {
          bytes += chunk.value.byteLength;
          if (bytes > this.storageLimitBytes) throw new Error("Dropbox track exceeds local storage limit");
          await file.writeFile(chunk.value);
          chunk = await reader.read();
        }
      } finally {
        await reader.cancel();
        await file.close();
      }
      await this.makeRoom(target, (await stat(temporary)).size);
      await rename(temporary, target);
      return { ok: true, value: target };
    } catch {
      return downloadFailure();
    } finally {
      await rm(temporary, { force: true }).catch(() => {});
    }
  }

  private async makeRoom(target: string, newBytes: number): Promise<void> {
    const files = await Promise.all((await readdir(this.directory))
      .filter((name) => !name.endsWith(".part"))
      .map(async (name) => {
        const path = join(this.directory, name);
        const info = await stat(path);
        return { path, size: info.size, modified: info.mtimeMs };
      }));
    let total = files.filter((file) => file.path !== target).reduce((sum, file) => sum + file.size, newBytes);
    for (const file of files.sort((a, b) => a.modified - b.modified)) {
      if (total <= this.storageLimitBytes) break;
      if (file.path === target) continue;
      await rm(file.path);
      total -= file.size;
    }
  }

}

function downloadFailure(): Result<never> {
  return failure("remote", "Could not download this Dropbox track. Check the network and reconnect Dropbox if access was revoked.");
}
