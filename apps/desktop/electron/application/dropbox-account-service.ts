import { randomBytes } from "node:crypto";
import type { DropboxConnectionStatus } from "../../shared/dropbox-contracts";
import { failure, type Result } from "../../shared/result";
import { buildDropboxAuthorizationUrl } from "./dropbox-oauth";
import type { DropboxAccountRepository } from "./interfaces/dropbox-account-interfaces";
import type { DropboxAuthorizationClient } from "./interfaces/dropbox-authorization-interfaces";

const authorizationLifetimeSeconds = 600;

export class DropboxAccountService {
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
    const pending = this.pending;
    if (!pending || pending.expiresAt <= this.now() || !code.trim()) {
      return failure("authorization", "Start a new Dropbox connection and enter its authorization code.");
    }
    try {
      const token = await this.createClient(pending.clientId).exchangeAuthorizationCode({
        authorizationCode: code.trim(), codeVerifier: pending.codeVerifier,
      });
      if (!token.refreshToken || !token.accountId || !token.accessToken || !Number.isFinite(token.expiresIn) || token.expiresIn <= 0) {
        return failure("authorization", "Dropbox did not grant offline access. Connect again.");
      }
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
    this.pending = null;
    try {
      await this.accounts.disconnect();
      return { ok: true, value: { connected: false, accountId: null } };
    } catch {
      return failure("storage", "Could not disconnect Dropbox. Try again.");
    }
  }
}
