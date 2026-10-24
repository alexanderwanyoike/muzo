// @vitest-environment node
import { mkdtempSync, readFileSync, readdirSync, rmSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { expect, it, vi } from "vitest";
import { DropboxAudioDownloader } from "./dropbox-audio-downloader";

it("downloads binary audio atomically without exposing credentials or remote filenames on disk", async () => {
  const root = mkdtempSync(join(tmpdir(), "muzo-cloud-audio-"));
  try {
    const request = vi.fn<typeof fetch>().mockResolvedValue(new Response(new Uint8Array([0, 255, 12, 34])));
    const downloader = new DropboxAudioDownloader(root, request);
    const result = await downloader.download("secret", "lib-1:track-1", "/Music/夜.mp3");
    expect(result.ok).toBe(true);
    if (!result.ok) return;
    expect(readFileSync(result.value)).toEqual(Buffer.from([0, 255, 12, 34]));
    expect(readdirSync(root)).toHaveLength(1);
    expect(result.value).not.toContain("夜");
    const [, options] = request.mock.calls[0];
    expect(options?.headers).toMatchObject({ Authorization: "Bearer secret", "Dropbox-API-Arg": '{"path":"/Music/\\u591c.mp3"}' });
    request.mockResolvedValueOnce(new Response("denied secret", { status: 401 }));
    expect(await downloader.download("secret", "lib-1:track-2", "/other.mp3")).toEqual({ ok: false, error: { kind: "remote", message: "Could not download this Dropbox track. Check the network and reconnect Dropbox if access was revoked." } });
    expect(readdirSync(root)).toHaveLength(1);
  } finally { rmSync(root, { recursive: true, force: true }); }
});

it("bounds downloaded storage and removes partial files after an oversized response", async () => {
  const root = mkdtempSync(join(tmpdir(), "muzo-cloud-limit-"));
  try {
    const request = vi.fn<typeof fetch>().mockImplementation(async () => new Response(new Uint8Array([1, 2, 3])));
    const downloader = new DropboxAudioDownloader(root, request, 4);
    expect((await downloader.download("token", "first", "/first.mp3")).ok).toBe(true);
    const second = await downloader.download("token", "second", "/second.mp3");
    expect(second.ok).toBe(true);
    expect(readdirSync(root)).toHaveLength(1);
    request.mockResolvedValueOnce(new Response(new Uint8Array([1, 2, 3, 4, 5])));
    expect((await downloader.download("token", "oversize", "/large.mp3")).ok).toBe(false);
    expect(readdirSync(root)).toHaveLength(1);
  } finally { rmSync(root, { recursive: true, force: true }); }
});

it("keeps the most recently requested track when an older download completes late", async () => {
  const root = mkdtempSync(join(tmpdir(), "muzo-cloud-concurrent-"));
  try {
    let finishOld: (response: Response) => void = () => {};
    const delayed = new Promise<Response>((resolve) => { finishOld = resolve; });
    const request = vi.fn<typeof fetch>().mockReturnValueOnce(delayed).mockResolvedValueOnce(new Response(new Uint8Array([4, 5, 6])));
    const downloader = new DropboxAudioDownloader(root, request, 4);
    const old = downloader.download("token", "old", "/old.mp3");
    const current = await downloader.download("token", "current", "/current.mp3");
    expect(current.ok).toBe(true);
    finishOld(new Response(new Uint8Array([1, 2, 3])));
    expect((await old).ok).toBe(false);
    if (current.ok) expect(readFileSync(current.value)).toEqual(Buffer.from([4, 5, 6]));
  } finally { rmSync(root, { recursive: true, force: true }); }
});
