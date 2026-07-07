import type { RuntimeBridge } from "./runtime-bridge";

export function installTestRuntime(runtime: RuntimeBridge): RuntimeBridge {
  window.__MUZO_RUNTIME__ = runtime;
  return runtime;
}

export function removeTestRuntime(): void {
  delete window.__MUZO_RUNTIME__;
}
