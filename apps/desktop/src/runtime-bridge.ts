import { invoke as tauriInvoke } from "@tauri-apps/api/core";
import { open as tauriOpen } from "@tauri-apps/plugin-dialog";

export interface RuntimeBridge {
  invoke<T>(command: string, args?: unknown): Promise<T>;
  openDirectory(): Promise<string | null>;
}

declare global {
  interface Window {
    __MUZO_RUNTIME__?: RuntimeBridge;
  }
}

const tauriRuntime: RuntimeBridge = {
  invoke: (command, args) =>
    tauriInvoke(command, args as Parameters<typeof tauriInvoke>[1]),
  openDirectory: async () => {
    const selected = await tauriOpen({ directory: true, multiple: false });
    return typeof selected === "string" ? selected : null;
  },
};

export function getRuntimeBridge(): RuntimeBridge {
  return window.__MUZO_RUNTIME__ ?? tauriRuntime;
}

export function invokeCommand<T>(command: string, args?: unknown): Promise<T> {
  return getRuntimeBridge().invoke<T>(command, args);
}

export function openDirectory(): Promise<string | null> {
  return getRuntimeBridge().openDirectory();
}
