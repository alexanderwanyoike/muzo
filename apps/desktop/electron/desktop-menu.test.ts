// @vitest-environment node
import { expect, it, vi } from "vitest";
import { installDesktopMenu } from "./desktop-menu";

it.each(["linux", "win32"] as const)(
  "removes the default window menu on %s instead of hiding it behind Alt",
  (platform) => {
    const menu = { setApplicationMenu: vi.fn(), buildFromTemplate: vi.fn() };
    installDesktopMenu(menu, platform);
    expect(menu.setApplicationMenu).toHaveBeenCalledWith(null);
    expect(menu.buildFromTemplate).not.toHaveBeenCalled();
  },
);

it("keeps only macOS app controls and hidden native editing shortcuts", () => {
  const installed = { identity: "native-menu" };
  const menu = {
    setApplicationMenu: vi.fn(),
    buildFromTemplate: vi.fn().mockReturnValue(installed),
  };
  installDesktopMenu(menu, "darwin");
  const template = menu.buildFromTemplate.mock.calls[0]?.[0] as {
    label: string;
    submenu: {
      role: string;
      visible?: boolean;
      accelerator?: string;
      acceleratorWorksWhenHidden?: boolean;
    }[];
  }[];
  expect(template?.map((item) => item.label)).toEqual(["Muzo"]);
  expect(menu.setApplicationMenu).toHaveBeenCalledWith(installed);
  expect(template[0].submenu).toContainEqual(
    expect.objectContaining({ role: "quit" }),
  );
  for (const role of ["undo", "redo", "cut", "copy", "paste", "selectAll"]) {
    expect(template[0].submenu).toContainEqual(
      expect.objectContaining({
        role,
        visible: false,
        acceleratorWorksWhenHidden: true,
      }),
    );
  }
});
