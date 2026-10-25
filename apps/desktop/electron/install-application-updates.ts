import type { IpcMain, IpcMainInvokeEvent } from "electron";
import type { createApplicationUpdates } from "./application/application-updates";
import { failure, success } from "./application/result";
import type { UpdateStatus } from "../shared/update-status";

interface UpdateWindow {
  isDestroyed(): boolean;
  webContents: {
    isDestroyed(): boolean;
    send(channel: string, status: UpdateStatus): void;
  };
}
export function installApplicationUpdates(
  ipc: Pick<IpcMain, "handle">,
  updates: ReturnType<typeof createApplicationUpdates>,
  findWindow: (event: IpcMainInvokeEvent) => UpdateWindow | null,
  allWindows: () => UpdateWindow[],
) {
  function trusted(event: IpcMainInvokeEvent) {
    const window = findWindow(event);
    return (
      event.senderFrame === event.sender.mainFrame &&
      window &&
      !window.isDestroyed()
    );
  }
  ipc.handle("muzo:get-update-status", (event) =>
    trusted(event)
      ? success(updates.getStatus())
      : failure("authorization", "The player window is unavailable."),
  );
  ipc.handle(
    "muzo:update-command",
    (event, command: unknown, ...extra: unknown[]) => {
      if (!trusted(event))
        return failure(
          "authorization",
          "Only the player window can manage updates.",
        );
      if (extra.length)
        return failure(
          "invalidInput",
          "Update commands do not accept arguments.",
        );
      switch (command) {
        case "check":
          return updates.check();
        case "download":
          return updates.download();
        case "install":
          return updates.install();
        default:
          return failure("invalidInput", "Unknown update command.");
      }
    },
  );
  return updates.subscribe((status) => {
    for (const window of allWindows()) {
      if (!window.isDestroyed() && !window.webContents.isDestroyed())
        window.webContents.send("muzo:update-status", status);
    }
  });
}
