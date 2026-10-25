import type { App, Dialog, Shell } from "electron";
import electronUpdater from "electron-updater";
import { createApplicationUpdates } from "./application/application-updates";
import { failure, success } from "./application/result";
import { createNativeUpdateBackend } from "./infrastructure/updates/native-update-backend";
import { latestMuzoRelease } from "./infrastructure/updates/github-update-release";
import { replaceAppImage } from "./infrastructure/updates/replace-appimage";
import { MUZO_UPDATE_FEED } from "../shared/update-feed";

const CHECK_INTERVAL_MS = 6 * 60 * 60 * 1000;
interface Options {
  app: Pick<App, "isPackaged" | "getVersion" | "relaunch" | "quit">;
  dialog: Pick<Dialog, "showMessageBox">;
  shell: Pick<Shell, "openExternal">;
  platform: NodeJS.Platform;
  arch: string;
  appImage?: string;
}
export function createInstalledUpdates({
  app,
  dialog,
  shell,
  platform,
  arch,
  appImage,
}: Options) {
  const automatic =
    app.isPackaged &&
    arch === "x64" &&
    (platform === "win32" || (platform === "linux" && Boolean(appImage)));
  const backend = createNativeUpdateBackend({
    enabled: app.isPackaged,
    automatic,
    createClient: () => {
      return platform === "linux"
        ? new electronUpdater.AppImageUpdater(MUZO_UPDATE_FEED)
        : new electronUpdater.NsisUpdater(MUZO_UPDATE_FEED);
    },
    checkRelease: latestMuzoRelease,
    openRelease: (url) => shell.openExternal(url),
    async confirmRestart(version) {
      const result = await dialog.showMessageBox({
        type: "question",
        message: `Restart Muzo to install v${version}?`,
        detail:
          "Playback will stop while Muzo installs the update and reopens.",
        buttons: ["Later", "Restart to install"],
        defaultId: 0,
        cancelId: 0,
        noLink: true,
      });
      return result.response === 1;
    },
    async installDownloaded(path, client) {
      if (platform === "linux" && appImage) {
        const replaced = await replaceAppImage(appImage, path);
        if (!replaced.ok) return replaced;
        try {
          app.relaunch({ execPath: appImage });
          app.quit();
        } catch {
          return failure(
            "storage",
            "The update is installed. Close Muzo and reopen the AppImage to use it.",
          );
        }
      } else client.quitAndInstall(true, true);
      return success(undefined);
    },
  });
  const updates = createApplicationUpdates({
    installedVersion: app.getVersion(),
    backend,
  });
  let timer: ReturnType<typeof setInterval> | undefined;
  return {
    updates,
    start() {
      if (!app.isPackaged || timer) return;
      void updates.check();
      timer = setInterval(() => {
        void updates.check();
      }, CHECK_INTERVAL_MS);
      timer.unref();
    },
    stop() {
      if (timer) clearInterval(timer);
      timer = undefined;
      updates.dispose();
    },
  };
}
