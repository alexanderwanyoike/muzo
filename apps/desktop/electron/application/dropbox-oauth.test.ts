// @vitest-environment node

import { describe, expect, it } from "vitest";

import { buildDropboxAuthorizationUrl } from "./dropbox-oauth";

describe("Dropbox OAuth", () => {
  it("builds a PKCE authorization URL for offline Dropbox access", () => {
    const url = new URL(
      buildDropboxAuthorizationUrl({
        clientId: "app-key",
        redirectUri: "muzo://dropbox-auth",
        scopes: ["files.metadata.read"],
        state: "state-123",
        codeVerifier:
          "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789-._~",
      }),
    );

    expect(url.origin).toBe("https://www.dropbox.com");
    expect(url.pathname).toBe("/oauth2/authorize");
    expect(url.searchParams.get("client_id")).toBe("app-key");
    expect(url.searchParams.get("redirect_uri")).toBe("muzo://dropbox-auth");
    expect(url.searchParams.get("response_type")).toBe("code");
    expect(url.searchParams.get("token_access_type")).toBe("offline");
    expect(url.searchParams.get("code_challenge_method")).toBe("S256");
    expect(url.searchParams.get("code_challenge")).toBe(
      "ImpiCd8pp4MveCNnbIS7-GXEtB0xF5HMIDoWqvGA5ig",
    );
    expect(url.searchParams.get("state")).toBe("state-123");
    expect(url.searchParams.get("scope")).toBe("files.metadata.read");
  });
});
