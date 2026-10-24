import { createRoot } from "react-dom/client";
import { saveSidebarWidth } from "../../src/sidebar-layout";
import App from "../../src/App";
import {
  applyAppearance,
  saveAppearance,
  type Appearance,
} from "../../src/appearance";
import "@fontsource-variable/manrope";
import "../../src/themes.css";
import "../../src/styles.css";

const parameters = new URLSearchParams(window.location.search);
const connected = parameters.get("connected") !== "false";
const appearance: Appearance = {
  theme:
    parameters.get("theme") === "paper"
      ? "paper"
      : parameters.get("theme") === "studio"
        ? "studio"
        : "aluminium",
  mode: parameters.get("mode") === "dark" ? "dark" : "light",
  accent: "cobalt",
};
const saved = saveAppearance(appearance);
if (!saved.ok) throw new Error(saved.error.message);
applyAppearance(appearance);
const sidebar = saveSidebarWidth(360);
if (!sidebar.ok) throw new Error(sidebar.error.message);
const libraries = [
  {
    id: "local",
    name: "Evening listening",
    kind: "filesystem",
    location: `/Music/${"Long album collection/".repeat(8)}`,
  },
];
function response(command: string): unknown {
  if (command === "list_libraries") return libraries;
  if (command === "dropbox_status")
    return {
      ok: true,
      value: { connected, accountId: connected ? "fixture" : null },
    };
  if (command === "dropbox_sync_status")
    return {
      ok: true,
      value: {
        intervalMinutes: 5,
        syncing: false,
        lastError: null,
        lastSyncedAtUnixSeconds: null,
      },
    };
  if (command === "dropbox_list_folders")
    return {
      ok: true,
      value: {
        folders: [{ id: "music", name: "Music", path: "/Music" }],
        addedPaths: [],
      },
    };
  return [];
}
window.__MUZO_RUNTIME__ = {
  async invoke<T>(command: string): Promise<T> {
    return response(command) as T;
  },
  async openDirectory() {
    return null;
  },
};
const root = document.getElementById("root");
if (!root) throw new Error("Settings fixture root required");
createRoot(root).render(<App />);
