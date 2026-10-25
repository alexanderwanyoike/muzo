// @vitest-environment node
import { expect, it } from "vitest";
import { launchAppImage } from "./launch-appimage";

it("returns a readable failure when the installed AppImage cannot be started", async () => {
  const result = await launchAppImage("/muzo-missing/update.AppImage", []);
  expect(result).toEqual({
    ok: false,
    error: {
      kind: "storage",
      message:
        "The update is installed, but Muzo could not restart. Open /muzo-missing/update.AppImage to continue.",
    },
  });
});

it("handles a rejected executable path without throwing", async () => {
  expect((await launchAppImage("invalid\0path", [])).ok).toBe(false);
});
