import { failure, success, type Result } from "./result";
import { newerStableVersion } from "./update-version";
import type { UpdateStatus } from "../../shared/update-status";
import type { UpdateBackend } from "./interfaces/update-backend";

interface Options {
  installedVersion: string;
  backend: UpdateBackend;
}

export function createApplicationUpdates({
  installedVersion,
  backend,
}: Options) {
  let status: UpdateStatus = { state: "idle" };
  let disposed = false;
  const listeners = new Set<(next: UpdateStatus) => void>();
  function publish(next: UpdateStatus) {
    if (disposed) return;
    status = next;
    for (const listener of listeners) listener({ ...status });
  }
  function fail(message: string): Result<never> {
    publish({ state: "failed", message });
    return failure("remote", message);
  }
  let pendingCheck: Promise<Result<void>> | null = null;
  function check(): Promise<Result<void>> {
    if (pendingCheck) return pendingCheck;
    if (
      status.state === "downloading" ||
      status.state === "ready" ||
      status.state === "installing"
    )
      return Promise.resolve(
        failure(
          "invalidInput",
          "Finish the current update before checking again.",
        ),
      );
    const operation = (async (): Promise<Result<void>> => {
      publish({ state: "checking" });
      const result = await backend.check();
      if (!result.ok) return fail(result.error.message);
      const newer = newerStableVersion(result.value.version, installedVersion);
      if (!newer.ok) return fail(newer.error.message);
      publish(
        newer.value
          ? { state: "available", ...result.value }
          : { state: "current" },
      );
      return success(undefined);
    })();
    pendingCheck = operation.finally(() => {
      pendingCheck = null;
    });
    return pendingCheck;
  }
  const unsubscribe = backend.subscribe((event) => {
    if (
      event.kind === "progress" &&
      status.state === "downloading" &&
      Number.isFinite(event.percent)
    )
      publish({
        ...status,
        progress: Math.min(100, Math.max(0, event.percent)),
      });
    if (event.kind === "installFailed" && status.state === "installing")
      publish({
        state: "failed",
        message:
          "The update could not be installed. Try again or download the latest release manually.",
      });
  });
  return {
    getStatus: (): UpdateStatus => ({ ...status }),
    dispose() {
      disposed = true;
      unsubscribe();
      listeners.clear();
    },
    subscribe(listener: (next: UpdateStatus) => void) {
      listeners.add(listener);
      return () => {
        listeners.delete(listener);
      };
    },
    check,
    async download(): Promise<Result<void>> {
      if (status.state !== "available")
        return failure("invalidInput", "No update is available to download.");
      const { version, automatic } = status;
      if (!automatic) return backend.openRelease(version);
      publish({ state: "downloading", version, progress: 0 });
      const result = await backend.download(version);
      if (!result.ok) return fail(result.error.message);
      publish({ state: "ready", version });
      return success(undefined);
    },
    async install(): Promise<Result<void>> {
      if (status.state !== "ready")
        return failure(
          "invalidInput",
          "Download and verify an update before restarting.",
        );
      const { version } = status;
      publish({ state: "installing", version });
      const result = await backend.install(version);
      if (!result.ok) return fail(result.error.message);
      if (result.value === "cancelled") publish({ state: "ready", version });
      return success(undefined);
    },
  };
}
