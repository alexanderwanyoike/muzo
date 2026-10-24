// @vitest-environment node
import { execFile } from "node:child_process";
import { mkdir, mkdtemp, readFile, readdir, rm, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { promisify } from "node:util";
import { describe, expect, it, vi } from "vitest";
import { registerAppImageMenuEntry } from "./appimage-menu-entry";

describe.skipIf(process.platform !== "linux")("Linux AppImage menu integration", () => {
it.skipIf(process.platform !== "linux")(
  "makes the portable player discoverable and launchable through the desktop menu",
  async () => {
    const directory = await mkdtemp(join(tmpdir(), "muzo-menu-entry-"));
    try {
      const launched = join(directory, "launched.json");
      const appImagePath = join(directory, "Muzo preview.AppImage");
      await writeFile(
        appImagePath,
        `#!${process.execPath}\nrequire('node:fs').writeFileSync(${JSON.stringify(launched)}, JSON.stringify(process.argv.slice(1)));\n`,
        { mode: 0o755 },
      );
      const dataDirectory = join(directory, "data");
      expect(
        await registerAppImageMenuEntry({
          appImagePath,
          dataDirectory,
          iconPath: join(import.meta.dirname, "../assets/icons/muzo.png"),
        }),
      ).toEqual({ ok: true, value: "registered" });
      await promisify(execFile)(
        "desktop-file-validate",
        [join(dataDirectory, "applications/muzo.desktop")],
      );
      const { stdout } = await promisify(execFile)(
        "/usr/bin/python3",
        [
          "-c",
          "from gi.repository import Gio; import json; app=Gio.DesktopAppInfo.new('muzo.desktop'); print(json.dumps({'name':app.get_name(),'class':app.get_startup_wm_class(),'icon':app.get_icon().get_file().get_path()})); app.launch([],None)",
        ],
        {
          env: {
            ...process.env,
            XDG_DATA_HOME: dataDirectory,
            XDG_CONFIG_HOME: join(directory, "config"),
          },
        },
      );
      expect(JSON.parse(stdout)).toEqual({
        name: "Muzo",
        class: "muzo",
        icon: join(dataDirectory, "muzo/appimage.png"),
      });
      expect(
        await readFile(join(dataDirectory, "muzo/appimage.png")),
      ).toEqual(
        await readFile(join(import.meta.dirname, "../assets/icons/muzo.png")),
      );
      await vi.waitFor(async () => {
        expect(JSON.parse(await readFile(launched, "utf8"))).toEqual([appImagePath]);
      });
    } finally {
      await rm(directory, { recursive: true, force: true });
    }
  },
);


it("preserves a launcher already installed by another package or integration tool", async () => {
  const directory = await mkdtemp(join(tmpdir(), "muzo-existing-menu-"));
  try {
    await mkdir(join(directory, "applications"));
    const launcher = join(directory, "applications/appimagekit-muzo.desktop");
    const entry = "[Desktop Entry]\nType=Application\nName=Muzo\nExec=/installed/muzo\nStartupWMClass=muzo\n";
    await writeFile(launcher, entry);
    expect(await registerAppImageMenuEntry({
      appImagePath: "/portable/Muzo.AppImage",
      dataDirectory: directory,
      iconPath: join(import.meta.dirname, "../assets/icons/muzo.png"),
    })).toEqual({ ok: true, value: "existing" });
    expect(await readFile(launcher, "utf8")).toBe(entry);
    await expect(readFile(join(directory, "applications/muzo.desktop"))).rejects.toMatchObject({ code: "ENOENT" });
  } finally {
    await rm(directory, { recursive: true, force: true });
  }
});


it("repairs Cinnamon's temporary-mount launcher without changing its pinned identity", async () => {
  const directory = await mkdtemp(join(tmpdir(), "muzo-cinnamon-menu-"));
  try {
    await mkdir(join(directory, "applications"));
    const filename = "Muzo.cinnamon-generated.desktop";
    const launcher = join(directory, "applications", filename);
    await writeFile(launcher, "[Desktop Entry]\nType=Application\nName=Muzo\nExec=/tmp/.mount_Muzo-old/muzo\nTerminal=false\n");
    const appImagePath = join(directory, "Applications/Muzo.AppImage");
    expect(await registerAppImageMenuEntry({
      appImagePath,
      dataDirectory: directory,
      iconPath: join(import.meta.dirname, "../assets/icons/muzo.png"),
    })).toEqual({ ok: true, value: "registered" });
    const entry = await readFile(launcher, "utf8");
    expect(entry).toContain(appImagePath);
    expect(entry).toContain("StartupWMClass=muzo");
    expect(entry).toContain("Icon=");
    expect(entry).not.toContain(".mount_");
    await expect(readFile(join(directory, "applications/muzo.desktop"))).rejects.toMatchObject({ code: "ENOENT" });
  } finally {
    await rm(directory, { recursive: true, force: true });
  }
});


it.each(["relative.AppImage", "/tmp/Muzo\nIcon=injected.AppImage", "/tmp/Muzo=preview.AppImage"])("rejects an invalid desktop executable path: %j", async (appImagePath) => {
  const directory = await mkdtemp(join(tmpdir(), "muzo-invalid-menu-"));
  try {
    const result = await registerAppImageMenuEntry({
      appImagePath, dataDirectory: directory,
      iconPath: join(import.meta.dirname, "../assets/icons/muzo.png"),
    });
    expect(result).toMatchObject({ ok: false, error: { kind: "invalidInput" } });
    expect(await readdir(directory)).toEqual([]);
  } finally {
    await rm(directory, { recursive: true, force: true });
  }
});

});
