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
  it("rejects expired sessions and prevents disconnect from reviving an in-flight connection", async () => {
    let stored: StoredDropboxAccount | null = null;
    let now = 1000;
    let resolveToken: (token: { accessToken: string; refreshToken: string; expiresIn: number; accountId: string }) => void = () => {};
    const token = new Promise<{ accessToken: string; refreshToken: string; expiresIn: number; accountId: string }>((resolve) => { resolveToken = resolve; });
    const service = new DropboxAccountService({ find: async () => stored, save: async (account) => { stored = account; }, disconnect: async () => { stored = null; } },
      () => ({ exchangeAuthorizationCode: () => token, refreshAccessToken: vi.fn() }), async () => {}, () => now);
    await service.begin("app-key");
    now += 601;
    expect(await service.complete("code")).toMatchObject({ ok: false, error: { kind: "authorization" } });
    await service.begin("app-key");
    const connecting = service.complete("code");
    await service.disconnect();
    resolveToken({ accessToken: "secret", refreshToken: "refresh", expiresIn: 3600, accountId: "account-1" });
    expect(await connecting).toMatchObject({ ok: false });
    expect(stored).toBeNull();
  });

  it("does not connect when authorization expires while token exchange is in flight", async () => {
    let now = 0;
    let finish: (token: { accessToken: string; refreshToken: string; expiresIn: number; accountId: string }) => void = () => {};
    const token = new Promise<{ accessToken: string; refreshToken: string; expiresIn: number; accountId: string }>((resolve) => { finish = resolve; });
    const save = vi.fn();
    const service = new DropboxAccountService({ find: async () => null, save, disconnect: async () => {} },
      () => ({ exchangeAuthorizationCode: () => token, refreshAccessToken: vi.fn() }), async () => {}, () => now);
    await service.begin("app-key");
    const connecting = service.complete("code");
    now = 601;
    finish({ accessToken: "secret", refreshToken: "refresh", expiresIn: 3600, accountId: "account" });
    expect(await connecting).toMatchObject({ ok: false, error: { kind: "authorization" } });
    expect(save).not.toHaveBeenCalled();
  });

  it("refreshes expired credentials once for concurrent callers and retains the refresh token", async () => {
    let stored: StoredDropboxAccount = { accountId: "account-1", clientId: "app-key", accessToken: "expired", refreshToken: "refresh", accessTokenExpiresAtUnixSeconds: 999, connectedAtUnixSeconds: 1 };
    const refreshAccessToken = vi.fn(async () => ({ accessToken: "fresh", refreshToken: null, expiresIn: 3600, accountId: null }));
    const service = new DropboxAccountService({ find: async () => stored, save: async (account) => { stored = account; }, disconnect: async () => {} },
      () => ({ exchangeAuthorizationCode: vi.fn(), refreshAccessToken }), async () => {}, () => 1000);
    expect(await Promise.all([service.accessToken(), service.accessToken()])).toEqual([{ ok: true, value: "fresh" }, { ok: true, value: "fresh" }]);
    expect(refreshAccessToken).toHaveBeenCalledTimes(1);
    expect(stored).toMatchObject({ refreshToken: "refresh", accessTokenExpiresAtUnixSeconds: 4600 });
    expect(await service.accessToken()).toEqual({ ok: true, value: "fresh" });
    expect(refreshAccessToken).toHaveBeenCalledTimes(1);
  });

});
