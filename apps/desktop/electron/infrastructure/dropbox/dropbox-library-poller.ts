import type { DropboxSyncStatus } from "../../../shared/dropbox-contracts";
import { failure, type Result } from "../../../shared/result";
import type { DropboxLibrarySettings } from "../../application/interfaces/dropbox-library-settings";

interface DropboxSyncTask {
  syncAll(): Promise<Result<{ librariesSynced: number; failures: string[] }>>;
}

const millisecondsPerMinute = 60_000;

export class DropboxLibraryPoller {
  private active = false;
  private syncing = false;
  private timer: ReturnType<typeof setTimeout> | null = null;
  private lastError: string | null = null;
  private lastSyncedAtUnixSeconds: number | null = null;

  constructor(
    private readonly task: DropboxSyncTask,
    private readonly settings: Pick<DropboxLibrarySettings, "intervalMinutes" | "setIntervalMinutes">,
    private readonly now: () => number,
  ) {}

  start(): void {
    if (this.active) return;
    this.active = true;
    void this.run();
  }

  stop(): void {
    this.active = false;
    if (this.timer) clearTimeout(this.timer);
    this.timer = null;
  }

  async status(): Promise<Result<DropboxSyncStatus>> {
    try {
      return { ok: true, value: {
        intervalMinutes: await this.settings.intervalMinutes(), syncing: this.syncing,
        lastError: this.lastError, lastSyncedAtUnixSeconds: this.lastSyncedAtUnixSeconds,
      } };
    } catch { return failure("storage", "Could not read Dropbox sync settings."); }
  }

  async setIntervalMinutes(minutes: number): Promise<Result<{ intervalMinutes: number }>> {
    if (!Number.isInteger(minutes) || minutes < 1 || minutes > 60) {
      return failure("invalidInput", "Choose a Dropbox sync interval between 1 and 60 minutes.");
    }
    try {
      await this.settings.setIntervalMinutes(minutes);
      if (this.timer) clearTimeout(this.timer);
      if (this.active && !this.syncing) this.schedule(minutes);
      return { ok: true, value: { intervalMinutes: minutes } };
    } catch { return failure("storage", "Could not save the Dropbox sync interval."); }
  }

  private async run(): Promise<void> {
    if (!this.active || this.syncing) return;
    this.syncing = true;
    try {
      const result = await this.task.syncAll();
      this.lastError = result.ok ? result.value.failures.join("; ") || null : result.error.message;
      if (!this.lastError) this.lastSyncedAtUnixSeconds = this.now();
    } catch {
      this.lastError = "Dropbox sync failed. It will retry automatically.";
    } finally {
      this.syncing = false;
      if (this.active) {
        let minutes = 5;
        try { minutes = await this.settings.intervalMinutes(); } catch { /* Retry even when settings cannot be read. */ }
        if (this.active) this.schedule(minutes);
      }
    }
  }

  private schedule(minutes: number): void {
    this.timer = setTimeout(() => { void this.run(); }, minutes * millisecondsPerMinute);
    this.timer.unref();
  }
}
