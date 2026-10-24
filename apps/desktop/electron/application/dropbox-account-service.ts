import { randomBytes } from "node:crypto";
import type { DropboxConnectionStatus } from "../../shared/dropbox-contracts";
import { failure, type Result } from "../../shared/result";
import { buildDropboxAuthorizationUrl } from "./dropbox-oauth";
import type { DropboxAccountRepository } from "./interfaces/dropbox-account-interfaces";
import type { DropboxAuthorizationClient } from "./interfaces/dropbox-authorization-interfaces";

const authorizationLifetimeSeconds = 600;

export class DropboxAccountService {
  private revision = 0;

  private refreshing: Promise<Result<{ accountId: string; accessToken: string }>> | null = null;

  private pending: { clientId: string; codeVerifier: string; expiresAt: number } | null = null;

  constructor(
    private readonly accounts: DropboxAccountRepository,
    private readonly createClient: (clientId: string) => DropboxAuthorizationClient,
    private readonly openExternal: (url: string) => Promise<void>,
    private readonly now: () => number,
  ) {}

  async begin(clientId: string): Promise<Result<{ pending: true }>> {
    if (!/^[a-zA-Z0-9_-]+$/.test(clientId)) {
      return failure("invalidInput", "Enter a valid Dropbox app key.");
    }
    const codeVerifier = randomBytes(32).toString("base64url");
    this.pending = { clientId, codeVerifier, expiresAt: this.now() + authorizationLifetimeSeconds };
    try {
      await this.openExternal(buildDropboxAuthorizationUrl({
        clientId, codeVerifier, state: randomBytes(32).toString("base64url"),
        scopes: ["files.metadata.read", "files.content.read"],
      }));
      return { ok: true, value: { pending: true } };
    } catch {
      this.pending = null;
      return failure("authorization", "Could not open Dropbox authorization. Try again.");
    }
  }

  async complete(code: string): Promise<Result<DropboxConnectionStatus>> {
    const {pending} = this;
    if (!pending || pending.expiresAt <= this.now() || !code.trim()) {
      return failure("authorization", "Start a new Dropbox connection and enter its authorization code.");
    }
    try {
      const token = await this.createClient(pending.clientId).exchangeAuthorizationCode({
        authorizationCode: code.trim(), codeVerifier: pending.codeVerifier,
      });
      if (this.pending !== pending || pending.expiresAt <= this.now()) {
        if (this.pending === pending) this.pending = null;
        return failure("authorization", "This Dropbox connection expired or was cancelled. Start again.");
      }
      if (!token.refreshToken || !token.accountId || !token.accessToken || !Number.isFinite(token.expiresIn) || token.expiresIn <= 0) {
        return failure("authorization", "Dropbox did not grant offline access. Connect again.");
      }
      this.pending = null;
      this.revision += 1;
      this.refreshing = null;
      await this.accounts.save({
        clientId: pending.clientId, accountId: token.accountId,
        accessToken: token.accessToken, refreshToken: token.refreshToken,
        accessTokenExpiresAtUnixSeconds: this.now() + token.expiresIn,
        connectedAtUnixSeconds: this.now(),
      });
      this.pending = null;
      return { ok: true, value: { connected: true, accountId: token.accountId } };
    } catch {
      return failure("authorization", "Could not connect Dropbox. Check the code, network, and secure storage, then try again.");
    }
  }

  async status(): Promise<Result<DropboxConnectionStatus>> {
    try {
      const account = await this.accounts.find();
      return { ok: true, value: { connected: account !== null, accountId: account?.accountId ?? null } };
    } catch {
      return failure("storage", "Could not read Dropbox credentials from secure storage.");
    }
  }

  async disconnect(): Promise<Result<DropboxConnectionStatus>> {
    this.revision += 1;
    this.refreshing = null;
    this.pending = null;
    try {
      await this.accounts.disconnect();
      return { ok: true, value: { connected: false, accountId: null } };
    } catch {
      return failure("storage", "Could not disconnect Dropbox. Try again.");
    }
  }

  async accessToken(expectedAccountId?: string): Promise<Result<string>> {
    this.refreshing ??= this.loadAccessToken(this.revision);
    const request = this.refreshing;
    try {
      const result = await request;
      if (!result.ok) return result;
      if (expectedAccountId && expectedAccountId !== result.value.accountId) {
        return failure("notConnected", "Connect the Dropbox account that owns this library.");
      }
      return { ok: true, value: result.value.accessToken };
    } finally {
      if (this.refreshing === request) this.refreshing = null;
    }
  }

  private async loadAccessToken(revision: number): Promise<Result<{ accountId: string; accessToken: string }>> {
    try {
      const account = await this.accounts.find();
      if (!account || !account.clientId || revision !== this.revision) {
        return failure("notConnected", "Connect Dropbox in Settings first.");
      }
      const refreshMarginSeconds = 60;
      if (account.accessTokenExpiresAtUnixSeconds > this.now() + refreshMarginSeconds) {
        return { ok: true, value: { accountId: account.accountId, accessToken: account.accessToken } };
      }
      const token = await this.createClient(account.clientId).refreshAccessToken(account.refreshToken);
      if (revision !== this.revision) return failure("notConnected", "Dropbox was disconnected.");
      if (!token.accessToken || !Number.isFinite(token.expiresIn) || token.expiresIn <= 0) {
        return failure("authorization", "Dropbox credentials could not be refreshed. Connect again in Settings.");
      }
      await this.accounts.save({
        ...account, accessToken: token.accessToken,
        refreshToken: token.refreshToken ?? account.refreshToken,
        accessTokenExpiresAtUnixSeconds: this.now() + token.expiresIn,
      });
      return { ok: true, value: { accountId: account.accountId, accessToken: token.accessToken } };
    } catch {
      return failure("authorization", "Could not refresh Dropbox access. Check the network or reconnect in Settings.");
    }
  }
}
