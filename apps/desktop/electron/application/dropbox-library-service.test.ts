// @vitest-environment node
import { expect, it, vi } from "vitest";
import { DropboxLibraryService } from "./dropbox-library-service";
import type { LibraryDto } from "../../src/types";

it("registers and syncs a Dropbox library using main-process credentials", async () => {
  const libraries: LibraryDto[] = [];
  const bindings = new Map<string, string>();
  const scan = vi.fn(async () => ({ tracksScanned: 2 }));
  const service = new DropboxLibraryService({
    add: async (library) => { libraries.push(library); }, list: async () => libraries,
    findById: async (id) => libraries.find((library) => library.id === id) ?? null,
  }, { status: async () => ({ ok: true, value: { connected: true, accountId: "account-1" } }), accessToken: async () => ({ ok: true, value: "main-process-token" }) },
  { bind: async (id, accountId) => { bindings.set(id, accountId); }, accountFor: async (id) => bindings.get(id) ?? null,
    intervalMinutes: async () => 5, setIntervalMinutes: async () => {} }, { scan }, () => "lib-1");

  expect(await service.add({ name: "Cloud Music", location: "/Music/" })).toEqual({ ok: true, value: { id: "lib-1", name: "Cloud Music", kind: "dropbox", location: "/Music" } });
  expect(await service.sync("lib-1")).toEqual({ ok: true, value: { tracksScanned: 2 } });
  expect(scan).toHaveBeenCalledWith({ libraryId: "lib-1", accessToken: "main-process-token" });
  bindings.set("lib-1", "other-account");
  expect(await service.sync("lib-1")).toMatchObject({ ok: false, error: { kind: "notConnected" } });
  expect(scan).toHaveBeenCalledTimes(1);
  expect(await service.add({ name: "Bad", location: "../../music" })).toMatchObject({ ok: false, error: { kind: "invalidInput" } });
});
