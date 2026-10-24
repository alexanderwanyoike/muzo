import type { DropboxFolder } from "../../application/interfaces/dropbox-folder-interfaces";

const dropboxTokenUrl = "https://api.dropboxapi.com/oauth2/token";
const dropboxListFolderUrl = "https://api.dropboxapi.com/2/files/list_folder";
const dropboxListFolderContinueUrl =
  "https://api.dropboxapi.com/2/files/list_folder/continue";

export interface DropboxClientConfig {
  clientId: string;
}

export interface DropboxHttpRequest {
  method: string;
  url: string;
  headers: [string, string][];
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
    let { cursor } = firstPage;
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

  async listFolders(
    accessToken: string,
    path: string,
  ): Promise<DropboxFolder[]> {
    const firstPage = await this.postJson<DropboxListFolderResponse>(
      accessToken,
      dropboxListFolderUrl,
      {
        path: path === "/" ? "" : path,
        recursive: false,
        include_deleted: false,
      },
    );
    const entries = [...firstPage.entries];
    let { cursor } = firstPage;
    let hasMore = firstPage.has_more;
    while (hasMore) {
      const page = await this.postJson<DropboxListFolderResponse>(
        accessToken,
        dropboxListFolderContinueUrl,
        { cursor },
      );
      entries.push(...page.entries);
      cursor = page.cursor;
      hasMore = page.has_more;
    }
    return entries
      .filter((entry) => entry[".tag"] === "folder")
      .map((entry) => ({
        id: entry.id,
        name: entry.name,
        path: entry.path_display ?? entry.path_lower ?? "",
      }));
  }

  private async postTokenForm(body: string): Promise<DropboxToken> {
    const response = await this.transport.send({
      method: "POST",
      url: dropboxTokenUrl,
      headers: [["Content-Type", "application/x-www-form-urlencoded"]],
      body,
    });
    assertOk(response);
    const payload: unknown = JSON.parse(response.body);
    return dropboxTokenFromResponse(payload);
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

export { FetchDropboxHttpTransport } from "./fetch-dropbox-http-transport";

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
    throw Object.assign(new Error(`Dropbox returned HTTP ${response.status}`), {
      kind: "unexpectedStatus",
      status: response.status,
      body: response.body,
    });
  }
}

function dropboxTokenFromResponse(response: unknown): DropboxToken {
  if (
    !response ||
    typeof response !== "object" ||
    !("access_token" in response) ||
    typeof response.access_token !== "string" ||
    !response.access_token ||
    !("expires_in" in response) ||
    typeof response.expires_in !== "number" ||
    !Number.isFinite(response.expires_in) ||
    response.expires_in <= 0
  ) {
    throw new Error("Invalid Dropbox token response");
  }
  const refreshToken =
    "refresh_token" in response ? response.refresh_token : undefined;
  const accountId = "account_id" in response ? response.account_id : undefined;
  if (
    (refreshToken !== undefined && typeof refreshToken !== "string") ||
    (accountId !== undefined && typeof accountId !== "string")
  ) {
    throw new Error("Invalid Dropbox token response");
  }
  return {
    accessToken: response.access_token,
    refreshToken: refreshToken ?? null,
    expiresIn: response.expires_in,
    accountId: accountId ?? null,
  };
}

function dropboxEntryFromResponse(
  response: DropboxEntryResponse,
): DropboxEntry {
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
