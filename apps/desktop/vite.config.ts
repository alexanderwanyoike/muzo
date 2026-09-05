import { defineConfig } from "vite";
import type { PluginOption } from "vite";
import react from "@vitejs/plugin-react";

const port = Number(process.env.MUZO_DESKTOP_PORT ?? 1420);

export default defineConfig(() => ({
  plugins: react() as unknown as PluginOption[],

  // Vitest needs an explicit environment for DOM APIs.
  // `test` is consumed by vitest, ignored by plain `vite build`.
  clearScreen: false,
  server: {
    port,
    strictPort: true,
    host: false,
  },
  test: {
    environment: "jsdom",
    globals: true,
    include: ["src/**/*.{test,spec}.{ts,tsx}", "electron/**/*.{test,spec}.ts"],
  },
}));
