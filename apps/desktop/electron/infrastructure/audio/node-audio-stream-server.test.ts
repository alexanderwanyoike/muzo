// @vitest-environment node

import { writeFileSync } from "node:fs";
import { request } from "node:http";
import { join } from "node:path";
import { tmpdir } from "node:os";

import { afterEach, describe, expect, it } from "vitest";

import { NodeAudioStreamServer } from "./node-audio-stream-server";

describe("NodeAudioStreamServer", () => {
  const servers: NodeAudioStreamServer[] = [];

  afterEach(() => {
    for (const server of servers.splice(0)) {
      server.close();
    }
  });

  it("serves registered audio bytes with range support", async () => {
    const filePath = join(tmpdir(), `muzo-audio-${Date.now()}-${Math.random()}.mp3`);
    writeFileSync(filePath, "abcdef");
    const server = new NodeAudioStreamServer();
    servers.push(server);

    const url = await server.register(filePath, "audio/mpeg");

    await expect(get(url, { Range: "bytes=1-3" })).resolves.toEqual({
      statusCode: 206,
      contentType: "audio/mpeg",
      contentRange: "bytes 1-3/6",
      body: "bcd",
    });
  });
});

function get(
  url: string,
  headers: Record<string, string>,
): Promise<{
  statusCode: number | undefined;
  contentType: string | undefined;
  contentRange: string | undefined;
  body: string;
}> {
  return new Promise((resolve, reject) => {
    const req = request(url, { headers }, (response) => {
      const chunks: Buffer[] = [];
      response.on("data", (chunk: Buffer) => chunks.push(chunk));
      response.on("end", () => {
        resolve({
          statusCode: response.statusCode,
          contentType: response.headers["content-type"],
          contentRange: response.headers["content-range"],
          body: Buffer.concat(chunks).toString("utf8"),
        });
      });
    });
    req.on("error", reject);
    req.end();
  });
}
