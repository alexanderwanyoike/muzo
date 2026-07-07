export interface RuntimeBridge {
  invoke<T>(command: string, args?: unknown): Promise<T>;
  openDirectory(): Promise<string | null>;
}

declare global {
  interface Window {
    __MUZO_RUNTIME__?: RuntimeBridge;
  }
}

export function getRuntimeBridge(): RuntimeBridge {
  const runtime = window.__MUZO_RUNTIME__;
  if (!runtime) {
    throw new Error(
      "Muzo desktop runtime is not available. Start the app with yarn desktop:dev.",
    );
  }
  return runtime;
}

export async function invokeCommand<T>(
  command: string,
  args?: unknown,
): Promise<T> {
  return getRuntimeBridge().invoke<T>(command, args);
}

export async function openDirectory(): Promise<string | null> {
  return getRuntimeBridge().openDirectory();
}
