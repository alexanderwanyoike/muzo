import type { ReconciliationReport } from "./application/reconcile-filesystem-libraries-service";
import type { Logger } from "./application/interfaces/logger-interfaces";
import type { WatchFilesystemLibrariesReport } from "./application/watch-filesystem-libraries-service";

export interface StartupTasks {
  migrateDatabase: () => Promise<void>;
  reconcileFilesystemLibraries: () => Promise<ReconciliationReport>;
  watchFilesystemLibraries: () => Promise<WatchFilesystemLibrariesReport>;
  logger: Logger;
}

export async function runStartupTasks(tasks: StartupTasks): Promise<void> {
  await tasks.migrateDatabase();
  const report = await tasks.reconcileFilesystemLibraries();
  for (const failure of report.failures) {
    tasks.logger.error(
      `filesystem library reconciliation failed for ${failure.libraryId}: ${failure.message}`,
    );
  }
  const watchReport = await tasks.watchFilesystemLibraries();
  for (const failure of watchReport.failures) {
    tasks.logger.error(
      `filesystem library watcher failed for ${failure.libraryId}: ${failure.message}`,
    );
  }
}
