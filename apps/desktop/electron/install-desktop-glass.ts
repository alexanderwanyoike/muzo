import type { IpcMain, IpcMainInvokeEvent } from "electron";
import { failure } from "./application/result";
import { setDesktopGlass, type DesktopGlassEnvironment } from "./desktop-glass";

type GlassWindow = Parameters<typeof setDesktopGlass>[0];

export function installDesktopGlass(
  ipc: Pick<IpcMain, "handle">,
  findWindow: (event: IpcMainInvokeEvent) => GlassWindow | null,
  environment: () => DesktopGlassEnvironment,
) {
  ipc.handle("muzo:desktop-glass", (event, enabled: unknown) => {
    if (event.senderFrame !== event.sender.mainFrame)
      return failure(
        "authorization",
        "Only the player window can change desktop glass.",
      );
    if (typeof enabled !== "boolean")
      return failure("invalidInput", "Unknown desktop glass setting.");
    const window = findWindow(event);
    if (!window || window.isDestroyed())
      return failure("remote", "The player window is unavailable.");
    return setDesktopGlass(window, enabled, environment());
  });
}
