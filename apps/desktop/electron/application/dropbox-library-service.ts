import type { LibraryRecord } from "./interfaces/library-record";
import type { DropboxConnectionStatus } from "../../shared/dropbox-contracts";
import { failure, type Result } from "../../shared/result";
import type { DropboxLibrarySettings } from "./interfaces/dropbox-library-settings";
import type { LibraryRepository } from "./interfaces/repository-interfaces";
import type { DropboxScanReport, ScanDropboxLibraryInput } from "./scan-dropbox-library-service";

export interface DropboxLibraryAccess {
  status(): Promise<Result<DropboxConnectionStatus>>;
  accessToken(expectedAccountId: string): Promise<Result<string>>;
}

export class DropboxLibraryService {
  private readonly inFlight = new Map<string, Promise<Result<DropboxScanReport>>>();
  constructor(
    private readonly libraries: LibraryRepository,
    private readonly account: DropboxLibraryAccess,
    private readonly settings: DropboxLibrarySettings,
    private readonly scanner: { scan(input: ScanDropboxLibraryInput): Promise<Result<DropboxScanReport>> },
    private readonly generateId: () => string,
  ) {}

  async add(input: { name: string; location: string }): Promise<Result<LibraryRecord>> {
    const name = input.name.trim();
    const location = input.location.trim().replace(/\/+$/, "") || "/";
    if (!name || !input.location.trim() || !location.startsWith("/") || location.split("/").some((part) => part === "." || part === "..")) {
      return failure("invalidInput", "Enter a library name and an absolute Dropbox folder path, such as /Music or /.");
    }
    const status = await this.account.status();
    if (!status.ok) return status;
    if (!status.value.accountId) return failure("notConnected", "Connect Dropbox in Settings first.");
    try {
      const library: LibraryRecord = { id: this.generateId(), name, kind: "dropbox", location };
      await this.libraries.add(library);
      await this.settings.bind(library.id, status.value.accountId);
      return { ok: true, value: library };
    } catch {
      return failure("storage", "Could not save the Dropbox library.");
    }
  }

  async accessFor(libraryId: string): Promise<Result<{ accessToken: string; library: LibraryRecord }>> {
    try {
      const library = await this.libraries.findById(libraryId);
      if (!library || library.kind !== "dropbox") return failure("invalidInput", "Select a Dropbox library.");
      const status = await this.account.status();
      if (!status.ok) return status;
      if (!status.value.accountId || await this.settings.accountFor(libraryId) !== status.value.accountId) {
        return failure("notConnected", "Connect the Dropbox account that owns this library.");
      }
      const token = await this.account.accessToken(status.value.accountId);
      return token.ok ? { ok: true, value: { accessToken: token.value, library } } : token;
    } catch {
      return failure("storage", "Could not load the Dropbox library settings.");
    }
  }

  sync(libraryId: string): Promise<Result<DropboxScanReport>> {
    const existing = this.inFlight.get(libraryId);
    if (existing) return existing;
    const request = this.syncOnce(libraryId).finally(() => { this.inFlight.delete(libraryId); });
    this.inFlight.set(libraryId, request);
    return request;
  }

  private async syncOnce(libraryId: string): Promise<Result<DropboxScanReport>> {
    const access = await this.accessFor(libraryId);
    if (!access.ok) return access;
    try {
      return await this.scanner.scan({ libraryId, accessToken: access.value.accessToken });
    } catch {
      return failure("remote", "Could not sync Dropbox. Check the folder path, network, or account connection and try again.");
    }
  }

  async syncAll(): Promise<Result<{ librariesSynced: number; failures: string[] }>> {
    try {
      let librariesSynced = 0;
      const failures: string[] = [];
      for (const library of await this.libraries.list()) {
        if (library.kind !== "dropbox") continue;
        const result = await this.sync(library.id);
        if (result.ok) librariesSynced += 1;
        else failures.push(`${library.name}: ${result.error.message}`);
      }
      return { ok: true, value: { librariesSynced, failures } };
    } catch {
      return failure("storage", "Could not list Dropbox libraries for sync.");
    }
  }
}
