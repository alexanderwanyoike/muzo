// @vitest-environment node
import { describe, expect, it, vi } from "vitest";
import type { StoredDropboxAccount } from "./interfaces/dropbox-account-interfaces";
import { DropboxAccountService } from "./dropbox-account-service";

describe("Dropbox account connection", () => {
  it("connects through PKCE and persists offline credentials without returning secrets", async () => {
    let stored: StoredDropboxAccount | null = null;
    const openExternal = vi.fn<(url: string) => Promise<void>>().mockResolvedValue(undefined);
    const exchangeAuthorizationCode = vi.fn(async () => ({
      accessToken: "secret-access", refreshToken: "secret-refresh",
      expiresIn: 3600, accountId: "account-1",
    }));
    const service = new DropboxAccountService({
      find: async () => stored,
      save: async (account) => { stored = account; },
      disconnect: async () => { stored = null; },
    }, () => ({ exchangeAuthorizationCode, refreshAccessToken: vi.fn() }),
    openExternal, () => 1000);

    expect(await service.begin("app-key")).toEqual({ ok: true, value: { pending: true } });
    const url = new URL(openExternal.mock.calls[0][0]);
    expect(url.origin).toBe("https://www.dropbox.com");
    expect(url.searchParams.get("code_challenge_method")).toBe("S256");
    expect(url.searchParams.get("token_access_type")).toBe("offline");
    expect(await service.complete("returned-code")).toEqual({
      ok: true, value: { connected: true, accountId: "account-1" },
    });
    expect(exchangeAuthorizationCode).toHaveBeenCalledWith({
      authorizationCode: "returned-code", codeVerifier: expect.stringMatching(/^[A-Za-z0-9_-]{43,128}$/),
    });
    expect(stored).toMatchObject({ clientId: "app-key", accessToken: "secret-access", refreshToken: "secret-refresh", accessTokenExpiresAtUnixSeconds: 4600 });
    expect(await service.status()).toEqual({ ok: true, value: { connected: true, accountId: "account-1" } });
    expect(await service.disconnect()).toEqual({ ok: true, value: { connected: false, accountId: null } });
    expect(stored).toBeNull();
  });
});
