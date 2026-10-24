// @vitest-environment node
import { expect, it, vi } from "vitest";
import { TrackArtworkService } from "./track-artwork-service";

it("reads Dropbox artwork with account-bound credentials without returning them to the renderer", async () => {
  const source = { libraryId: "cloud", trackId: "track", path: "/Music/song.flac", kind: "dropbox" as const, revision: "1:2" };
  const read = vi.fn().mockResolvedValue({ format: "image/png", data: new Uint8Array([1]) });
  const accessFor = vi.fn().mockResolvedValue({ ok: true, value: { accessToken: "private-token", library: { id: "cloud", kind: "dropbox", location: "/Music", name: "Music" } } });
  const service = new TrackArtworkService({ find: async () => source }, { read: vi.fn() }, { create: () => "data:image/png;base64,cover" }, { read }, { accessFor });
  expect(await service.get("cloud", "track")).toEqual({ ok: true, value: "data:image/png;base64,cover" });
  expect(accessFor).toHaveBeenCalledWith("cloud");
  expect(read).toHaveBeenCalledWith("private-token", "/Music/song.flac");
  accessFor.mockResolvedValueOnce({ ok: false, error: { kind: "notConnected", message: "Connect the owning account." } });
  expect(await service.get("cloud", "track")).toMatchObject({ ok: false, error: { kind: "notConnected" } });
  expect(read).toHaveBeenCalledOnce();
});
