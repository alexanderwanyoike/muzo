// @vitest-environment node
import { existsSync } from "node:fs";
import { dirname } from "node:path";
import { fileURLToPath } from "node:url";
import { expect, it } from "vitest";
import { configureDesktopIcon } from "./desktop-icon";

const electronDir = dirname(fileURLToPath(import.meta.url));
it.each([["linux", "png"], ["win32", "ico"]] as const)("uses a shipped %s window icon", (platform, extension) => {
  const path = configureDesktopIcon({ isPackaged: false }, electronDir, platform);
  expect(path.endsWith(`muzo.${extension}`)).toBe(true);
  expect(existsSync(path)).toBe(true);
});
