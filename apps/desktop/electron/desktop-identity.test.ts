// @vitest-environment node
import { join } from "node:path";
import { expect, it, vi } from "vitest";
import { configureDesktopIdentity } from "./desktop-identity";

it("keeps the established credential namespace until Electron initializes secure storage", () => {
  let ready: (() => void) | undefined;
  const app = { setName: vi.fn(), setPath: vi.fn(), getPath: vi.fn(() => "/profile-root"), once: vi.fn((_event: string, listener: () => void) => { ready = listener; }) };
  configureDesktopIdentity(app);
  expect(app.setName.mock.calls).toEqual([["Electron"]]);
  expect(app.setPath).toHaveBeenCalledWith("userData", join("/profile-root", "Muzo"));
  expect(app.setPath).toHaveBeenCalledWith("sessionData", join("/profile-root", "Muzo"));
  expect(app.once).toHaveBeenCalledWith("ready", expect.any(Function));
  ready?.();
  expect(app.setName.mock.calls).toEqual([["Electron"], ["Muzo"]]);
});
