// @vitest-environment node

import { join } from "node:path";
import { tmpdir } from "node:os";

import { describe, expect, it } from "vitest";

import { ElectronContainer } from "./electron-container";

describe("ElectronContainer", () => {
  it("wires command handlers with infrastructure dependencies", async () => {
    const container = new ElectronContainer({
      dbPath: tempDatabasePath(),
      generateLibraryId: () => "lib-1",
    });

    await expect(
      container.commandDispatcher.handleElectronCommand("add_library", {
        input: {
          name: "Local Music",
          kind: "filesystem",
          location: "/music",
        },
      }),
    ).resolves.toEqual({
      id: "lib-1",
      name: "Local Music",
      kind: "filesystem",
      location: "/music",
    });

    await expect(
      container.commandDispatcher.handleElectronCommand("list_libraries"),
    ).resolves.toEqual([
      {
        id: "lib-1",
        name: "Local Music",
        kind: "filesystem",
        location: "/music",
      },
    ]);
  });
});

function tempDatabasePath(): string {
  return join(tmpdir(), `muzo-${Date.now()}-${Math.random()}.sqlite`);
}
