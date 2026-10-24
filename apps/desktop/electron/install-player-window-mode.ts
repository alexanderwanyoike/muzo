import type { BrowserWindow, IpcMain, IpcMainInvokeEvent } from "electron";
import { failure } from "./application/result";
import { createPlayerWindowModeController } from "./player-window-mode";

type PlayerWindow = Parameters<typeof createPlayerWindowModeController>[0];

export function installPlayerWindowMode(
  ipc: Pick<IpcMain, "handle">,
  findWindow: (
    event: IpcMainInvokeEvent,
  ) => PlayerWindow | BrowserWindow | null,
  nativeCorners = false,
) {
  const controllers = new WeakMap<
    PlayerWindow,
    ReturnType<typeof createPlayerWindowModeController>
  >();
  ipc.handle("muzo:player-window-mode", (event, mode: unknown) => {
    if (event.senderFrame !== event.sender.mainFrame)
      return failure(
        "authorization",
        "Only the player window can change its mode.",
      );
    const window = findWindow(event);
    if (!window) return failure("remote", "The player window is unavailable.");
    let controller = controllers.get(window);
    if (!controller) {
      controller = createPlayerWindowModeController(window, undefined, undefined, nativeCorners);
      controllers.set(window, controller);
    }
    return controller.setMode(mode);
  });
}
