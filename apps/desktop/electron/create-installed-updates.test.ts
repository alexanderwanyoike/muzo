// @vitest-environment node
import { EventEmitter } from "node:events";
import { expect, it, vi } from "vitest";
import { success } from "./application/result";
import { createInstalledUpdates } from "./create-installed-updates";

const native = vi.hoisted(() => ({
  check: vi.fn(),
  download: vi.fn(),
  install: vi.fn(),
  create: vi.fn(),
  replace: vi.fn(),
  release: vi.fn(),
  spawn: vi.fn(),
}));
vi.mock("node:child_process", () => ({ spawn: native.spawn }));
vi.mock("electron-updater", () => ({
  default: {
    AppImageUpdater: class {
      constructor() {
        native.create();
      }

      checkForUpdates = native.check;

      downloadUpdate = native.download;

      quitAndInstall = native.install;

      on = vi.fn();
    },
    NsisUpdater: class {
      constructor() {
        native.create();
      }

      checkForUpdates = native.check;

      downloadUpdate = native.download;

      quitAndInstall = native.install;

      on = vi.fn();
    },
  },
}));
vi.mock("./infrastructure/updates/github-update-release", () => ({
  latestMuzoRelease: native.release,
  MUZO_RELEASES_URL: "https://github.com/alexanderwanyoike/muzo/releases",
}));
vi.mock("./infrastructure/updates/replace-appimage", () => ({
  replaceAppImage: native.replace,
}));
it("checks installed builds only and replaces a Linux AppImage at its existing path after explicit restart", async () => {
  native.release.mockResolvedValue(success({ version: "0.0.3" }));
  native.check.mockResolvedValue({ updateInfo: { version: "0.0.3" } });
  native.download.mockResolvedValue(["/verified.AppImage"]);
  native.replace.mockResolvedValue(success(undefined));
  const app = {
    isPackaged: true,
    getVersion: () => "0.0.2",
    relaunch: vi.fn(),
    once: vi.fn(),
    quit: vi.fn(),
  };
  const dialog = {
    showErrorBox: vi.fn(),
    showMessageBox: vi.fn(async () => ({
      response: 1,
      checkboxChecked: false,
    })),
  };
  const shell = { openExternal: vi.fn(async () => {}) };
  const runtime = createInstalledUpdates({
    app,
    dialog,
    shell,
    platform: "linux",
    arch: "x64",
    appImage: "/Applications/Muzo.AppImage",
  });
  await runtime.updates.check();
  await runtime.updates.download();
  expect(app.quit).not.toHaveBeenCalled();
  expect(native.replace).not.toHaveBeenCalled();
  await runtime.updates.install();
  expect(native.replace).toHaveBeenCalledWith(
    "/Applications/Muzo.AppImage",
    "/verified.AppImage",
  );
  expect(app.relaunch).not.toHaveBeenCalled();
  expect(native.spawn).not.toHaveBeenCalled();
  expect(app.once).toHaveBeenCalledWith("will-quit", expect.any(Function));
  const child = Object.assign(new EventEmitter(), { unref: vi.fn() });
  native.spawn.mockReturnValue(child);
  const event = { preventDefault: vi.fn() };
  const onQuit = app.once.mock.calls[0]?.[1] as (event: {
    preventDefault(): void;
  }) => void;
  onQuit(event);
  expect(event.preventDefault).toHaveBeenCalledOnce();
  expect(native.spawn).toHaveBeenCalledWith(
    "/Applications/Muzo.AppImage",
    process.argv.slice(1),
    {
      cwd: "/Applications",
      detached: true,
      stdio: "ignore",
    },
  );
  child.emit("spawn");
  await vi.waitFor(() => expect(app.quit).toHaveBeenCalledTimes(2));
  expect(child.unref).toHaveBeenCalledOnce();
  expect(dialog.showErrorBox).not.toHaveBeenCalled();
  expect(app.quit).toHaveBeenCalledTimes(2);
  runtime.stop();
  native.release.mockClear();
  native.create.mockClear();
  const dev = createInstalledUpdates({
    app: { ...app, isPackaged: false },
    dialog,
    shell,
    platform: "linux",
    arch: "x64",
    appImage: "/wrong.AppImage",
  });
  dev.start();
  dev.stop();
  expect(native.release).not.toHaveBeenCalled();
  expect(native.create).not.toHaveBeenCalled();
  const mac = createInstalledUpdates({
    app,
    dialog,
    shell,
    platform: "darwin",
    arch: "arm64",
  });
  await mac.updates.check();
  expect(mac.updates.getStatus()).toMatchObject({ automatic: false });
  await mac.updates.download();
  expect(shell.openExternal).toHaveBeenCalledWith(
    "https://github.com/alexanderwanyoike/muzo/releases/tag/v0.0.3",
  );
  expect(native.create).not.toHaveBeenCalled();
  mac.stop();
});
