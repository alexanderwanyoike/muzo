import type { App } from "electron";
import { join } from "node:path";

const credentialStorageIdentity = "Electron";
const desktopName = "Muzo";
type IdentityApp = Pick<App, "setName" | "getPath" | "setPath" | "once">;

export function configureDesktopIdentity(app: IdentityApp): void {
  // Electron binds OS encryption keys before ready; existing tokens use this name.
  app.setName(credentialStorageIdentity);
  const profile = join(app.getPath("appData"), desktopName);
  app.setPath("userData", profile);
  app.setPath("sessionData", profile);
  app.once("ready", () => app.setName(desktopName));
}
