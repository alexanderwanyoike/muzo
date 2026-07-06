// @vitest-environment node

import { join } from "node:path";
import { tmpdir } from "node:os";
import { mkdirSync } from "node:fs";

import { describe, expect, it } from "vitest";

import { createElectronContainer } from "./electron-container";

describe("Electron container", () => {
  it("wires command handlers with infrastructure dependencies", async () => {
    const libraryRoot = tempDirectory();
    const container = createElectronContainer({
      dbPath: tempDatabasePath(),
      generateLibraryId: () => "lib-1",
    });
    const migrateDatabase = container.resolve("migrateDatabase");
    const commandDispatcher = container.resolve("commandDispatcher");

    await migrateDatabase();

    await expect(
      commandDispatcher.handleElectronCommand("add_library", {
        input: {
          name: "Local Music",
          kind: "filesystem",
          location: libraryRoot,
        },
      }),
    ).resolves.toEqual({
      id: "lib-1",
      name: "Local Music",
      kind: "filesystem",
      location: libraryRoot,
    });

    await expect(
      commandDispatcher.handleElectronCommand("scan_library", {
        input: { libraryId: "lib-1" },
      }),
    ).resolves.toEqual({ tracksScanned: 0 });

    await expect(
      commandDispatcher.handleElectronCommand("list_libraries"),
    ).resolves.toEqual([
      {
        id: "lib-1",
        name: "Local Music",
        kind: "filesystem",
        location: libraryRoot,
      },
    ]);
  });
});

function tempDatabasePath(): string {
  return join(tmpdir(), `muzo-${Date.now()}-${Math.random()}.sqlite`);
}

function tempDirectory(): string {
  const directory = join(tmpdir(), `muzo-${Date.now()}-${Math.random()}`);
  mkdirSync(directory, { recursive: true });
  return directory;
}
