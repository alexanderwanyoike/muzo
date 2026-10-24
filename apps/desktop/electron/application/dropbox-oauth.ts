import { createHash } from "node:crypto";

const dropboxAuthorizeUrl = "https://www.dropbox.com/oauth2/authorize";

export interface DropboxAuthorizationRequest {
  clientId: string;
  redirectUri?: string;
  scopes: string[];
  state: string;
  codeVerifier: string;
}

export function buildDropboxAuthorizationUrl(
  request: DropboxAuthorizationRequest,
): string {
  const url = new URL(dropboxAuthorizeUrl);
  url.searchParams.set("client_id", request.clientId);
  url.searchParams.set("response_type", "code");
  url.searchParams.set("token_access_type", "offline");
  url.searchParams.set("code_challenge_method", "S256");
  url.searchParams.set("code_challenge", pkceS256Challenge(request.codeVerifier));
  url.searchParams.set("state", request.state);

  if (request.redirectUri) {
    url.searchParams.set("redirect_uri", request.redirectUri);
  }
  if (request.scopes.length > 0) {
    url.searchParams.set("scope", request.scopes.join(" "));
  }

  return url.toString();
}

function pkceS256Challenge(codeVerifier: string): string {
  return createHash("sha256")
    .update(codeVerifier)
    .digest("base64url");
}
