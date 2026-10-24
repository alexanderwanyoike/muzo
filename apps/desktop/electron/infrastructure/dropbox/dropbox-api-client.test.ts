// @vitest-environment node

import { afterEach, describe, expect, it, vi } from "vitest";

import {
  DropboxApiClient,
  FetchDropboxHttpTransport,
  type DropboxHttpRequest,
  type DropboxHttpResponse,
  type DropboxHttpTransport,
} from "./dropbox-api-client";

describe("DropboxApiClient", () => {
  afterEach(() => {
    vi.restoreAllMocks();
  });

  it("exchanges an authorization code for a Dropbox token", async () => {
    const transport = new FakeTransport([
      {
        status: 200,
        body: JSON.stringify({
          access_token: "access-123",
          refresh_token: "refresh-123",
          expires_in: 14400,
          account_id: "dbid:123",
          token_type: "bearer",
        }),
      },
    ]);

    const token = await new DropboxApiClient(
      { clientId: "app-key" },
      transport,
    ).exchangeAuthorizationCode({
      authorizationCode: "auth-code",
      codeVerifier:
        "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789-._~",
      redirectUri: "muzo://dropbox-auth",
    });

    expect(token).toEqual({
      accessToken: "access-123",
      refreshToken: "refresh-123",
      expiresIn: 14400,
      accountId: "dbid:123",
    });
    expect(transport.requests).toHaveLength(1);
    expect(transport.requests[0]).toMatchObject({
      method: "POST",
      url: "https://api.dropboxapi.com/oauth2/token",
      headers: [["Content-Type", "application/x-www-form-urlencoded"]],
    });
    expect(formFromBody(transport.requests[0].body)).toEqual({
      grant_type: "authorization_code",
      client_id: "app-key",
      code: "auth-code",
      code_verifier:
        "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789-._~",
      redirect_uri: "muzo://dropbox-auth",
    });
  });

  it("refreshes a Dropbox access token", async () => {
    const transport = new FakeTransport([
      {
        status: 200,
        body: JSON.stringify({
          access_token: "access-123",
          expires_in: 14400,
        }),
      },
    ]);

    const token = await new DropboxApiClient(
      { clientId: "app-key" },
      transport,
    ).refreshAccessToken("refresh-123");

    expect(token.accessToken).toBe("access-123");
    expect(formFromBody(transport.requests[0].body)).toEqual({
      grant_type: "refresh_token",
      client_id: "app-key",
      refresh_token: "refresh-123",
    });
  });

  it("lists a Dropbox folder recursively across pages", async () => {
    const transport = new FakeTransport([
      {
        status: 200,
        body: JSON.stringify({
          entries: [
            {
              ".tag": "folder",
              name: "Album",
              path_lower: "/music/album",
              path_display: "/Music/Album",
              id: "id:folder",
            },
          ],
          cursor: "cursor-1",
          has_more: true,
        }),
      },
      {
        status: 200,
        body: JSON.stringify({
          entries: [
            {
              ".tag": "file",
              name: "Song.mp3",
              path_lower: "/music/album/song.mp3",
              path_display: "/Music/Album/Song.mp3",
              id: "id:file",
              server_modified: "2026-06-25T10:00:00Z",
              rev: "rev-1",
              size: 12345,
              content_hash: "hash-1",
            },
          ],
          cursor: "cursor-2",
          has_more: false,
        }),
      },
    ]);

    const entries = await new DropboxApiClient(
      { clientId: "app-key" },
      transport,
    ).listFolderRecursive("access-123", "/Music");

    expect(entries).toEqual([
      {
        tag: "folder",
        name: "Album",
        id: "id:folder",
        pathLower: "/music/album",
        pathDisplay: "/Music/Album",
        serverModified: null,
        rev: null,
        size: null,
        contentHash: null,
      },
      {
        tag: "file",
        name: "Song.mp3",
        id: "id:file",
        pathLower: "/music/album/song.mp3",
        pathDisplay: "/Music/Album/Song.mp3",
        serverModified: "2026-06-25T10:00:00Z",
        rev: "rev-1",
        size: 12345,
        contentHash: "hash-1",
      },
    ]);
    expect(transport.requests[0]).toMatchObject({
      url: "https://api.dropboxapi.com/2/files/list_folder",
      headers: [
        ["Authorization", "Bearer access-123"],
        ["Content-Type", "application/json"],
      ],
    });
    expect(JSON.parse(transport.requests[0].body)).toEqual({
      path: "/Music",
      recursive: true,
      include_deleted: false,
    });
    expect(transport.requests[1].url).toBe(
      "https://api.dropboxapi.com/2/files/list_folder/continue",
    );
    expect(JSON.parse(transport.requests[1].body)).toEqual({
      cursor: "cursor-1",
    });
  });

  it("sends HTTP requests with fetch", async () => {
    const fetchSpy = vi.spyOn(globalThis, "fetch").mockResolvedValue({
      status: 200,
      text: async () => '{"done":true}\n',
    } as Response);

    const response = await new FetchDropboxHttpTransport().send({
      method: "POST",
      url: "https://api.dropboxapi.com/oauth2/token",
      headers: [
        ["Authorization", "Bearer access-123"],
        ["Content-Type", "application/json"],
      ],
      body: '{"ok":true}',
    });

    expect(fetchSpy).toHaveBeenCalledWith(
      "https://api.dropboxapi.com/oauth2/token",
      {
        method: "POST",
        headers: {
          Authorization: "Bearer access-123",
          "Content-Type": "application/json",
        },
        body: '{"ok":true}',
      },
    );
    expect(response).toEqual({
      status: 200,
      body: '{"done":true}\n',
    });
  });
});

class FakeTransport implements DropboxHttpTransport {
  readonly requests: DropboxHttpRequest[] = [];

  private readonly responses: DropboxHttpResponse[];

  constructor(responses: DropboxHttpResponse[]) {
    this.responses = [...responses];
  }

  async send(request: DropboxHttpRequest): Promise<DropboxHttpResponse> {
    this.requests.push(request);
    const response = this.responses.shift();
    if (!response) {
      throw new Error("missing fake response");
    }
    return response;
  }
}

function formFromBody(body: string): Record<string, string> {
  return Object.fromEntries(new URLSearchParams(body));
}

it.each([
  null,
  {},
  { access_token: 42, expires_in: 3600 },
  { access_token: "token", expires_in: "3600" },
  { access_token: "token", expires_in: -1 },
  { access_token: "token", expires_in: 3600, refresh_token: 42 },
  { access_token: "token", expires_in: 3600, account_id: 42 },
])("rejects malformed Dropbox token data: %j", async (payload) => {
  const client = new DropboxApiClient(
    { clientId: "key" },
    new FakeTransport([{ status: 200, body: JSON.stringify(payload) }]),
  );
  await expect(client.refreshAccessToken("refresh")).rejects.toThrow(
    "Invalid Dropbox token response",
  );
});

it("rejects unsuccessful Dropbox HTTP responses with an error carrying status details", async () => {
  const client = new DropboxApiClient(
    { clientId: "key" },
    new FakeTransport([{ status: 401, body: "invalid credentials" }]),
  );
  const request = client.refreshAccessToken("refresh");
  await expect(request).rejects.toThrow("Dropbox returned HTTP 401");
  await expect(request).rejects.toMatchObject({
    kind: "unexpectedStatus",
    status: 401,
    body: "invalid credentials",
  });
});

it.each([
  { ".tag": "folder", id: "bad", name: "Missing path" },
  { ".tag": "folder", id: "bad", name: 42, path_display: "/Music" },
  { ".tag": "folder", id: null, name: "Music", path_display: "/Music" },
])(
  "rejects malformed folder metadata instead of showing an unselectable folder",
  async (entry) => {
    const client = new DropboxApiClient(
      { clientId: "key" },
      new FakeTransport([
        {
          status: 200,
          body: JSON.stringify({
            entries: [entry],
            cursor: "last",
            has_more: false,
          }),
        },
      ]),
    );
    await expect(client.listFolders("token", "/")).rejects.toThrow(
      "Invalid Dropbox folder metadata",
    );
  },
);

it("requests a temporary Dropbox content link for range metadata reads", async () => {
  const transport = new FakeTransport([{ status: 200, body: JSON.stringify({ link: "https://dl.dropboxusercontent.com/apitl/cover" }) }]);
  const client = new DropboxApiClient({ clientId: "" }, transport);
  expect(await client.temporaryLink("private-token", "/Music/song.flac")).toBe("https://dl.dropboxusercontent.com/apitl/cover");
  expect(transport.requests[0]).toMatchObject({ url: "https://api.dropboxapi.com/2/files/get_temporary_link", body: JSON.stringify({ path: "/Music/song.flac" }) });
});
