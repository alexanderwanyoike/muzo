import type { ReconciliationReport } from "./application/reconcile-filesystem-libraries-service";

export interface StartupTasks {
  migrateDatabase: () => Promise<void>;
  reconcileFilesystemLibraries: () => Promise<ReconciliationReport>;
  logError: (message: string) => void;
}

export async function runStartupTasks(tasks: StartupTasks): Promise<void> {
  await tasks.migrateDatabase();
  const report = await tasks.reconcileFilesystemLibraries();
  for (const failure of report.failures) {
    tasks.logError(
      `filesystem library reconciliation failed for ${failure.libraryId}: ${failure.message}`,
    );
  }
}
