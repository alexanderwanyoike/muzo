const dropboxTokenUrl = "https://api.dropboxapi.com/oauth2/token";
const dropboxListFolderUrl =
  "https://api.dropboxapi.com/2/files/list_folder";
const dropboxListFolderContinueUrl =
  "https://api.dropboxapi.com/2/files/list_folder/continue";

export interface DropboxClientConfig {
  clientId: string;
}

export interface DropboxHttpRequest {
  method: string;
  url: string;
  headers: Array<[string, string]>;
  body: string;
}

export interface DropboxHttpResponse {
  status: number;
  body: string;
}

export interface DropboxHttpTransport {
  send(request: DropboxHttpRequest): Promise<DropboxHttpResponse>;
}

export interface DropboxToken {
  accessToken: string;
  refreshToken: string | null;
  expiresIn: number;
  accountId: string | null;
}

export interface DropboxEntry {
  tag: "file" | "folder";
  name: string;
  id: string;
  pathLower: string | null;
  pathDisplay: string | null;
  serverModified: string | null;
  rev: string | null;
  size: number | null;
  contentHash: string | null;
}

export interface ExchangeAuthorizationCodeInput {
  authorizationCode: string;
  codeVerifier: string;
  redirectUri?: string;
}

export class DropboxApiClient {
  constructor(
    private readonly config: DropboxClientConfig,
    private readonly transport: DropboxHttpTransport,
  ) {}

  exchangeAuthorizationCode(
    input: ExchangeAuthorizationCodeInput,
  ): Promise<DropboxToken> {
    const form = new URLSearchParams({
      grant_type: "authorization_code",
      client_id: this.config.clientId,
      code: input.authorizationCode,
      code_verifier: input.codeVerifier,
    });
    if (input.redirectUri) {
      form.set("redirect_uri", input.redirectUri);
    }

    return this.postTokenForm(form.toString());
  }

  refreshAccessToken(refreshToken: string): Promise<DropboxToken> {
    const form = new URLSearchParams({
      grant_type: "refresh_token",
      client_id: this.config.clientId,
      refresh_token: refreshToken,
    });

    return this.postTokenForm(form.toString());
  }

  async listFolderRecursive(
    accessToken: string,
    path: string,
  ): Promise<DropboxEntry[]> {
    const firstPage = await this.postJson<DropboxListFolderResponse>(
      accessToken,
      dropboxListFolderUrl,
      {
        path,
        recursive: true,
        include_deleted: false,
      },
    );

    const entries = [...firstPage.entries];
    let cursor = firstPage.cursor;
    let hasMore = firstPage.has_more;

    while (hasMore) {
      const nextPage = await this.postJson<DropboxListFolderResponse>(
        accessToken,
        dropboxListFolderContinueUrl,
        { cursor },
      );
      entries.push(...nextPage.entries);
      cursor = nextPage.cursor;
      hasMore = nextPage.has_more;
    }

    return entries.map(dropboxEntryFromResponse);
  }

  private async postTokenForm(body: string): Promise<DropboxToken> {
    const response = await this.transport.send({
      method: "POST",
      url: dropboxTokenUrl,
      headers: [["Content-Type", "application/x-www-form-urlencoded"]],
      body,
    });
    assertOk(response);
    return dropboxTokenFromResponse(JSON.parse(response.body));
  }

  private async postJson<T>(
    accessToken: string,
    url: string,
    body: unknown,
  ): Promise<T> {
    const response = await this.transport.send({
      method: "POST",
      url,
      headers: [
        ["Authorization", `Bearer ${accessToken}`],
        ["Content-Type", "application/json"],
      ],
      body: JSON.stringify(body),
    });
    assertOk(response);
    return JSON.parse(response.body) as T;
  }
}

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

interface DropboxTokenResponse {
  access_token: string;
  refresh_token?: string;
  expires_in: number;
  account_id?: string;
}

interface DropboxListFolderResponse {
  entries: DropboxEntryResponse[];
  cursor: string;
  has_more: boolean;
}

interface DropboxEntryResponse {
  ".tag": "file" | "folder";
  name: string;
  id: string;
  path_lower?: string;
  path_display?: string;
  server_modified?: string;
  rev?: string;
  size?: number;
  content_hash?: string;
}

function assertOk(response: DropboxHttpResponse): void {
  if (response.status !== 200) {
    throw {
      kind: "unexpectedStatus",
      status: response.status,
      body: response.body,
    };
  }
}

function dropboxTokenFromResponse(response: DropboxTokenResponse): DropboxToken {
  return {
    accessToken: response.access_token,
    refreshToken: response.refresh_token ?? null,
    expiresIn: response.expires_in,
    accountId: response.account_id ?? null,
  };
}

function dropboxEntryFromResponse(response: DropboxEntryResponse): DropboxEntry {
  return {
    tag: response[".tag"],
    name: response.name,
    id: response.id,
    pathLower: response.path_lower ?? null,
    pathDisplay: response.path_display ?? null,
    serverModified: response.server_modified ?? null,
    rev: response.rev ?? null,
    size: response.size ?? null,
    contentHash: response.content_hash ?? null,
  };
}
