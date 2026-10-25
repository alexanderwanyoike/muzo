import type { App, Dialog } from "electron";
import { launchAppImage } from "./infrastructure/updates/launch-appimage";

export function scheduleAppImageRestart(
  app: Pick<App, "once" | "exit">,
  dialog: Pick<Dialog, "showErrorBox">,
  file: string,
): void {
  // Electron's relaunch helper lives in the temporary image that is about to unmount.
  app.once("will-quit", (event) => {
    event.preventDefault();
    void launchAppImage(file, process.argv.slice(1)).then((result) => {
      if (!result.ok)
        dialog.showErrorBox("Muzo could not restart", result.error.message);
      app.exit(0);
    });
  });
}
