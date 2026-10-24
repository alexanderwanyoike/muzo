// @vitest-environment node
import { existsSync } from "node:fs";
import { dirname } from "node:path";
import { fileURLToPath } from "node:url";
import { expect, it, vi } from "vitest";
import { configureDesktopIcon } from "./desktop-icon";

const electronDir = dirname(fileURLToPath(import.meta.url));
it.each([
  ["linux", "muzo-window.png"],
  ["win32", "muzo.ico"],
] as const)("uses a shipped %s window icon", (platform, extension) => {
  const path = configureDesktopIcon(
    { isPackaged: false },
    electronDir,
    platform,
  );
  expect(path.endsWith(extension)).toBe(true);
  expect(existsSync(path)).toBe(true);
});

it("sets the Muzo Dock icon during macOS development and preserves the packaged bundle icon", () => {
  const setIcon = vi.fn();
  const path = configureDesktopIcon(
    { isPackaged: false, dock: { setIcon } },
    electronDir,
    "darwin",
  );
  expect(setIcon).toHaveBeenCalledWith(path);
  setIcon.mockClear();
  configureDesktopIcon(
    { isPackaged: true, dock: { setIcon } },
    electronDir,
    "darwin",
  );
  configureDesktopIcon(
    { isPackaged: false, dock: { setIcon } },
    electronDir,
    "linux",
  );
  expect(setIcon).not.toHaveBeenCalled();
});
