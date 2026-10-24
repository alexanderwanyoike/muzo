// @vitest-environment node
import { expect, it, vi } from "vitest";
import { RemoveLibraryService } from "./remove-library-service";

it("keeps the watcher running when catalogue removal cannot be committed", async () => {
  const watcher = { unwatch: vi.fn() };
  const service = new RemoveLibraryService({ remove: vi.fn().mockRejectedValue(new Error("disk full")) }, watcher);
  expect(await service.remove("source")).toMatchObject({ ok: false, error: { kind: "repository" } });
  expect(watcher.unwatch).not.toHaveBeenCalled();
});

it("reports completed removal with a cleanup warning if closing the watcher fails", async () => {
  const service = new RemoveLibraryService({ remove: vi.fn().mockResolvedValue(true) }, { unwatch: vi.fn().mockRejectedValue(new Error("close failed")) });
  expect(await service.remove("source")).toEqual({ ok: true, value: { warning: "Source removed. Restart Muzo to release its filesystem watcher." } });
});
