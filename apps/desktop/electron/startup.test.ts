// @vitest-environment node

import { describe, expect, it, vi } from "vitest";

import { runStartupTasks } from "./startup";

describe("Electron startup", () => {
  it("runs migrations before filesystem reconciliation", async () => {
    const calls: string[] = [];

    await runStartupTasks({
      migrateDatabase: async () => {
        calls.push("migrate");
      },
      reconcileFilesystemLibraries: async () => {
        calls.push("reconcile");
        return { librariesReconciled: 1, failures: [] };
      },
      logError: vi.fn(),
    });

    expect(calls).toEqual(["migrate", "reconcile"]);
  });

  it("logs reconciliation failures and continues startup", async () => {
    const logError = vi.fn();

    await runStartupTasks({
      migrateDatabase: vi.fn().mockResolvedValue(undefined),
      reconcileFilesystemLibraries: vi.fn().mockResolvedValue({
        librariesReconciled: 0,
        failures: [{ libraryId: "lib-1", message: "not found" }],
      }),
      logError,
    });

    expect(logError).toHaveBeenCalledWith(
      "filesystem library reconciliation failed for lib-1: not found",
    );
  });
});
