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
      watchFilesystemLibraries: async () => {
        calls.push("watch");
        return { librariesWatched: 1, failures: [] };
      },
      logger: { error: vi.fn() },
    });

    expect(calls).toEqual(["migrate", "reconcile", "watch"]);
  });

  it("logs reconciliation failures and continues startup", async () => {
    const logger = { error: vi.fn() };

    await runStartupTasks({
      migrateDatabase: vi.fn().mockResolvedValue(undefined),
      reconcileFilesystemLibraries: vi.fn().mockResolvedValue({
        librariesReconciled: 0,
        failures: [{ libraryId: "lib-1", message: "not found" }],
      }),
      watchFilesystemLibraries: vi.fn().mockResolvedValue({
        librariesWatched: 0,
        failures: [],
      }),
      logger,
    });

    expect(logger.error).toHaveBeenCalledWith(
      "filesystem library reconciliation failed for lib-1: not found",
    );
  });

  it("logs watcher registration failures and continues startup", async () => {
    const logger = { error: vi.fn() };

    await runStartupTasks({
      migrateDatabase: vi.fn().mockResolvedValue(undefined),
      reconcileFilesystemLibraries: vi.fn().mockResolvedValue({
        librariesReconciled: 1,
        failures: [],
      }),
      watchFilesystemLibraries: vi.fn().mockResolvedValue({
        librariesWatched: 0,
        failures: [{ libraryId: "lib-1", message: "permission denied" }],
      }),
      logger,
    });

    expect(logger.error).toHaveBeenCalledWith(
      "filesystem library watcher failed for lib-1: permission denied",
    );
  });
});
