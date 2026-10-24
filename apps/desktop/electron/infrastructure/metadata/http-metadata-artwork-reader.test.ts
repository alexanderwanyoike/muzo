// @vitest-environment node
import { createServer } from "node:http";
import { readFileSync } from "node:fs";
import { once } from "node:events";
import { expect, it } from "vitest";
import { HttpMetadataArtworkReader } from "./http-metadata-artwork-reader";

it("reads an embedded remote cover using ranges without downloading the audio payload", async () => {
  const bytes = Buffer.concat([
    readFileSync(new URL("./fixtures/embedded-cover.flac", import.meta.url)),
    Buffer.alloc(8 * 1024 * 1024),
  ]);
  let transferred = 0;
  const server = createServer((request, response) => {
    const range = request.headers.range?.match(/^bytes=(\d+)-(\d+)$/);
    if (!range) {
      response.writeHead(400).end();
      return;
    }
    const first = Number(range[1]);
    const last = Math.min(Number(range[2]), bytes.length - 1);
    transferred += last - first + 1;
    response.writeHead(206, {
      "Content-Range": `bytes ${first}-${last}/${bytes.length}`,
      "Content-Length": last - first + 1,
      "Accept-Ranges": "bytes",
      "Content-Type": "audio/flac",
    });
    response.end(bytes.subarray(first, last + 1));
  });
  server.listen(0, "127.0.0.1");
  await once(server, "listening");
  try {
    const address = server.address();
    if (!address || typeof address === "string")
      throw new Error("Test server unavailable");
    const picture = await new HttpMetadataArtworkReader().read(
      `http://127.0.0.1:${address.port}/song.flac`,
    );
    expect(picture?.format).toBe("image/png");
    expect(Buffer.from(picture?.data ?? [])).toEqual(
      readFileSync(new URL("./fixtures/cover.png", import.meta.url)),
    );
    expect(transferred).toBeLessThan(128 * 1024);
  } finally {
    server.closeAllConnections();
    await new Promise<void>((resolve, reject) =>
      server.close((error) => (error ? reject(error) : resolve())),
    );
  }
});
