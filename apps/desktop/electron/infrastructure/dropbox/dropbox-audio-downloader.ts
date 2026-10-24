import { createHash, randomUUID } from "node:crypto";
import { mkdir, open, rename, rm } from "node:fs/promises";
import { join } from "node:path";
import { failure, type Result } from "../../../shared/result";
import type { DropboxAudioDownload } from "../../application/interfaces/dropbox-audio-download";

export class DropboxAudioDownloader implements DropboxAudioDownload {
  constructor(
    private readonly directory: string,
    private readonly request: typeof fetch = fetch,
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
        let chunk = await reader.read();
        while (!chunk.done) {
          await file.writeFile(chunk.value);
          chunk = await reader.read();
        }
      } finally {
        await reader.cancel();
        await file.close();
      }
      await rename(temporary, target);
      return { ok: true, value: target };
    } catch {
      return downloadFailure();
    } finally {
      await rm(temporary, { force: true }).catch(() => {});
    }
  }
}

function downloadFailure(): Result<never> {
  return failure("remote", "Could not download this Dropbox track. Check the network and reconnect Dropbox if access was revoked.");
}
