import type { Menu, MenuItemConstructorOptions } from "electron";

const hiddenShortcuts = ([
  { role: "undo", accelerator: "Cmd+Z" },
  { role: "redo", accelerator: "Shift+Cmd+Z" },
  { role: "cut", accelerator: "Cmd+X" },
  { role: "copy", accelerator: "Cmd+C" },
  { role: "paste", accelerator: "Cmd+V" },
  { role: "selectAll", accelerator: "Cmd+A" },
  { role: "close", accelerator: "Cmd+W" },
] satisfies MenuItemConstructorOptions[]).map((item) => ({
  ...item,
  visible: false,
  acceleratorWorksWhenHidden: true,
}));

export function installDesktopMenu(
  menu: Pick<typeof Menu, "setApplicationMenu" | "buildFromTemplate">,
  platform: NodeJS.Platform,
): void {
  if (platform !== "darwin") {
    menu.setApplicationMenu(null);
    return;
  }
  menu.setApplicationMenu(
    menu.buildFromTemplate([
      {
        label: "Muzo",
        submenu: [
          { role: "hide" },
          { role: "hideOthers" },
          { role: "unhide" },
          { type: "separator" },
          { role: "quit" },
          ...hiddenShortcuts,
        ],
      },
    ]),
  );
}
