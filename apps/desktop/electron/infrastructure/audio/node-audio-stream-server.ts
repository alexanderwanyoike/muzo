import { createReadStream, statSync } from "node:fs";
import { createServer, type Server, type IncomingMessage } from "node:http";
import { ulid } from "ulid";

import type { AudioSourceRegistry } from "../../application/interfaces/audio-source-interfaces";

interface AudioRoute {
  filePath: string;
  mimeType: string;
}

export class NodeAudioStreamServer implements AudioSourceRegistry {
  private readonly routes = new Map<string, AudioRoute>();
  private readonly server: Server;
  private readonly origin: Promise<string>;

  constructor() {
    this.server = createServer((request, response) => {
      void this.handle(request, response);
    });
    this.origin = new Promise((resolve, reject) => {
      this.server.once("error", reject);
      this.server.listen(0, "127.0.0.1", () => {
        this.server.unref();
        const address = this.server.address();
        if (!address || typeof address === "string") {
          reject(new Error("audio stream server did not bind to a TCP address"));
          return;
        }
        resolve(`http://127.0.0.1:${address.port}`);
      });
    });
  }

  close(): void {
    this.server.close();
  }

  async register(filePath: string, mimeType: string): Promise<string> {
    const token = ulid();
    this.routes.set(token, { filePath, mimeType });
    return `${await this.origin}/audio/${token}`;
  }

  private async handle(
    request: IncomingMessage,
    response: import("node:http").ServerResponse,
  ): Promise<void> {
    const token = request.url?.startsWith("/audio/")
      ? request.url.slice("/audio/".length)
      : null;
    const route = token ? this.routes.get(token) : null;
    if (!route) {
      response.writeHead(404, { "Content-Type": "text/plain" });
      response.end("not found");
      return;
    }

    const fileSize = statSync(route.filePath).size;
    const range = parseRange(request.headers.range, fileSize);
    const start = range?.start ?? 0;
    const end = range?.end ?? Math.max(fileSize - 1, 0);
    const statusCode = range ? 206 : 200;
    const headers: Record<string, string | number> = {
      "Accept-Ranges": "bytes",
      "Content-Length": Math.max(end - start + 1, 0),
      "Content-Type": route.mimeType,
    };
    if (range) {
      headers["Content-Range"] = `bytes ${start}-${end}/${fileSize}`;
    }

    response.writeHead(statusCode, headers);
    createReadStream(route.filePath, { start, end }).pipe(response);
  }
}

function parseRange(
  header: string | undefined,
  fileSize: number,
): { start: number; end: number } | null {
  if (!header?.startsWith("bytes=")) {
    return null;
  }

  const [startValue, endValue] = header.slice("bytes=".length).split("-", 2);
  const start = Number(startValue);
  const end = endValue ? Number(endValue) : fileSize - 1;
  if (
    !Number.isInteger(start) ||
    !Number.isInteger(end) ||
    start < 0 ||
    end < start ||
    end >= fileSize
  ) {
    return null;
  }
  return { start, end };
}
