import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";

// Tauri spawns a Vite dev server with a fixed port and strict host settings.
// See https://v2.tauri.app/start/frontend/vite/ for the reasoning.
const host = process.env.TAURI_DEV_HOST;

export default defineConfig(async () => ({
  plugins: [react()],

  // Vitest needs an explicit environment for DOM APIs.
  // `test` is consumed by vitest, ignored by plain `vite build`.
  clearScreen: false,
  server: {
    port: 1420,
    strictPort: true,
    host: host || false,
    hmr: host
      ? {
          protocol: "ws",
          host,
          port: 1421,
        }
      : undefined,
    watch: {
      ignored: ["**/src-tauri/**"],
    },
  },
  test: {
    environment: "jsdom",
    globals: true,
    include: ["src/**/*.{test,spec}.{ts,tsx}"],
  },
}));
