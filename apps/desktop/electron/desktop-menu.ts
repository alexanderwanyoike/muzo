import type { Menu } from "electron";

export function installDesktopMenu(menu: Pick<typeof Menu, "setApplicationMenu" | "buildFromTemplate">, _platform: NodeJS.Platform): void {
  menu.setApplicationMenu(null);
}
