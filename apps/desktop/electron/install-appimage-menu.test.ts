// @vitest-environment node
import { mkdtemp, readFile, readdir, rm } from "node:fs/promises";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { expect, it } from "vitest";
import { installAppImageMenu } from "./install-appimage-menu";

it.skipIf(process.platform !== "linux")("registers a packaged AppImage in the Linux desktop's configured data directory", async () => {
  const directory = await mkdtemp(join(tmpdir(), "muzo-appimage-startup-"));
  try {
    const data = join(directory, "custom data");
    expect(await installAppImageMenu({
      platform: "linux",
      isPackaged: true,
      homeDirectory: directory,
      iconPath: join(import.meta.dirname, "../assets/icons/muzo-window.png"),
      environment: { APPIMAGE: join(directory, "Muzo.AppImage"), XDG_DATA_HOME: data },
    })).toEqual({ ok: true, value: "registered" });
    expect(await readFile(join(data, "applications/muzo.desktop"), "utf8")).toContain(join(directory, "Muzo.AppImage"));
  } finally {
    await rm(directory, { recursive: true, force: true });
  }
});

it.each([
  { platform: "linux" as const, isPackaged: false, environment: { APPIMAGE: "/Muzo.AppImage" } },
  { platform: "linux" as const, isPackaged: true, environment: {} },
  { platform: "darwin" as const, isPackaged: true, environment: { APPIMAGE: "/Muzo.AppImage" } },
  { platform: "win32" as const, isPackaged: true, environment: { APPIMAGE: "/Muzo.AppImage" } },
])("leaves non-AppImage launches alone: %j", async (runtime) => {
  const directory = await mkdtemp(join(tmpdir(), "muzo-no-registration-"));
  try {
    expect(await installAppImageMenu({
      ...runtime,
      homeDirectory: directory,
      iconPath: "unused",
    })).toEqual({ ok: true, value: "skipped" });
    expect(await readdir(directory)).toEqual([]);
  } finally {
    await rm(directory, { recursive: true, force: true });
  }
});


it.skipIf(process.platform !== "linux").each([undefined, "relative-data"])("uses the XDG default when data home is absent or relative: %j", async (dataHome) => {
  const directory = await mkdtemp(join(tmpdir(), "muzo-default-data-"));
  try {
    expect(await installAppImageMenu({ platform: "linux", isPackaged: true, homeDirectory: directory, iconPath: join(import.meta.dirname, "../assets/icons/muzo-window.png"), environment: { APPIMAGE: join(directory, "Muzo.AppImage"), XDG_DATA_HOME: dataHome } })).toEqual({ ok: true, value: "registered" });
    expect(await readFile(join(directory, ".local/share/applications/muzo.desktop"), "utf8")).toContain("Name=Muzo");
  } finally {
    await rm(directory, { recursive: true, force: true });
  }
});
