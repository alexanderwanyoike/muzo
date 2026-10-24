// @vitest-environment node
import { expect, it, vi } from "vitest";
import { setDesktopGlass } from "./desktop-glass";

it.each(["linux", "darwin", "win32"] as const)(
  "reveals the desktop through the native %s surface and restores opacity",
  (platform) => {
    const window = {
      isDestroyed: () => false,
      setBackgroundColor: vi.fn(),
      setVibrancy: vi.fn(),
      setBackgroundMaterial: vi.fn(),
    };
    const environment = {
      platform,
      release: "10.0.22631",
      reducedTransparency: false,
    };
    expect(setDesktopGlass(window, true, environment)).toEqual({
      ok: true,
      value: true,
    });
    expect(window.setBackgroundColor).toHaveBeenLastCalledWith("#00000000");
    if (platform === "darwin")
      expect(window.setVibrancy).toHaveBeenLastCalledWith("under-window");
    if (platform === "win32")
      expect(window.setBackgroundMaterial).toHaveBeenLastCalledWith("acrylic");
    expect(setDesktopGlass(window, false, environment)).toEqual({
      ok: true,
      value: false,
    });
    expect(window.setBackgroundColor).toHaveBeenLastCalledWith("#ffffff");
    if (platform === "darwin")
      expect(window.setVibrancy).toHaveBeenLastCalledWith(null);
    if (platform === "win32")
      expect(window.setBackgroundMaterial).toHaveBeenLastCalledWith("none");
  },
);

it("keeps an unsupported Windows desktop opaque without calling its material API", () => {
  const window = {
    isDestroyed: () => false,
    setBackgroundColor: vi.fn(),
    setVibrancy: vi.fn(),
    setBackgroundMaterial: vi.fn(),
  };
  const environment = {
    platform: "win32" as const,
    release: "10.0.19045",
    reducedTransparency: false,
  };
  expect(setDesktopGlass(window, true, environment)).toMatchObject({
    ok: false,
    error: { kind: "remote" },
  });
  expect(window.setBackgroundMaterial).not.toHaveBeenCalled();
  expect(window.setBackgroundColor).not.toHaveBeenCalled();
  expect(setDesktopGlass(window, false, environment)).toEqual({
    ok: true,
    value: false,
  });
  expect(window.setBackgroundMaterial).not.toHaveBeenCalled();
  expect(window.setBackgroundColor).toHaveBeenCalledWith("#ffffff");
});

it("keeps the native surface opaque when the system reduces transparency", () => {
  const window = {
    isDestroyed: () => false,
    setBackgroundColor: vi.fn(),
    setVibrancy: vi.fn(),
    setBackgroundMaterial: vi.fn(),
  };
  expect(
    setDesktopGlass(window, true, {
      platform: "darwin",
      release: "25.0.0",
      reducedTransparency: true,
    }),
  ).toEqual({ ok: true, value: false });
  expect(window.setVibrancy).toHaveBeenCalledWith(null);
  expect(window.setBackgroundColor).toHaveBeenCalledWith("#ffffff");
});
