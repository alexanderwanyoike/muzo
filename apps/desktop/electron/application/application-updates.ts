import { failure, success, type Result } from "./result";
import type { UpdateStatus } from "../../shared/update-status";
import type { UpdateBackend } from "./interfaces/update-backend";

interface Options { installedVersion: string; backend: UpdateBackend }

export function createApplicationUpdates({ installedVersion, backend }: Options) {
  let status: UpdateStatus = { state: "idle" };
  const listeners = new Set<(next: UpdateStatus) => void>();
  function publish(next: UpdateStatus) {
    status = next;
    for (const listener of listeners) listener({ ...status });
  }
  function fail(message: string): Result<never> {
    publish({ state: "failed", message });
    return failure("remote", message);
  }
  return {
    getStatus: (): UpdateStatus => ({ ...status }),
    subscribe(listener: (next: UpdateStatus) => void) {
      listeners.add(listener);
      return () => { listeners.delete(listener); };
    },
    async check(): Promise<Result<void>> {
      publish({ state: "checking" });
      const result = await backend.check();
      if (!result.ok) return fail(result.error.message);
      publish(result.value.version === installedVersion ? { state: "current" } : { state: "available", ...result.value });
      return success(undefined);
    },
    async download(): Promise<Result<void>> {
      if (status.state !== "available") return failure("invalidInput", "No update is available to download.");
      const { version, automatic } = status;
      if (!automatic) return backend.openRelease(version);
      publish({ state: "downloading", version, progress: 0 });
      const result = await backend.download(version);
      if (!result.ok) return fail(result.error.message);
      publish({ state: "ready", version });
      return success(undefined);
    },
    async install(): Promise<Result<void>> {
      if (status.state !== "ready") return failure("invalidInput", "Download and verify an update before restarting.");
      const { version } = status;
      publish({ state: "installing", version });
      const result = await backend.install(version);
      if (!result.ok) return fail(result.error.message);
      if (result.value === "cancelled") publish({ state: "ready", version });
      return success(undefined);
    },
  };
}
