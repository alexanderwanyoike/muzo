import type { DropboxHttpTransport, DropboxHttpRequest, DropboxHttpResponse } from "./dropbox-api-client";

export class FetchDropboxHttpTransport implements DropboxHttpTransport {
  async send(request: DropboxHttpRequest): Promise<DropboxHttpResponse> {
    const response = await fetch(request.url, {
      method: request.method,
      headers: Object.fromEntries(request.headers),
      body: request.body,
    });
    return {
      status: response.status,
      body: await response.text(),
    };
  }
}

