// @vitest-environment node
import { expect, it, vi } from "vitest";
import { RemoveLibraryService } from "./remove-library-service";

it("keeps the watcher running when catalogue removal cannot be committed", async () => {
  const watcher = { unwatch: vi.fn() };
  const service = new RemoveLibraryService({ remove: vi.fn().mockRejectedValue(new Error("disk full")) }, watcher);
  expect(await service.remove("source")).toMatchObject({ ok: false, error: { kind: "repository" } });
  expect(watcher.unwatch).not.toHaveBeenCalled();
});
