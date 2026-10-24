// @vitest-environment node
import { expect, it, vi } from "vitest";
import { installDesktopMenu } from "./desktop-menu";

it.each(["linux", "win32"] as const)("removes the default window menu on %s instead of hiding it behind Alt", (platform) => {
  const menu = { setApplicationMenu: vi.fn(), buildFromTemplate: vi.fn() };
  installDesktopMenu(menu, platform);
  expect(menu.setApplicationMenu).toHaveBeenCalledWith(null);
  expect(menu.buildFromTemplate).not.toHaveBeenCalled();
});
