import type {
  UpdateBackend,
  UpdateBackendEvent,
} from "../../application/interfaces/update-backend";
import { failure, success, type Result } from "../../application/result";
import { MUZO_RELEASES_URL } from "./github-update-release";

export interface NativeUpdateClient {
  autoDownload: boolean;
  autoInstallOnAppQuit: boolean;
  allowPrerelease: boolean;
  allowDowngrade: boolean;
  disableDifferentialDownload: boolean;
  checkForUpdates(): Promise<{ updateInfo: { version: string } } | null>;
  downloadUpdate(): Promise<string[]>;
  quitAndInstall(isSilent?: boolean, isForceRunAfter?: boolean): void;
  on(
    event: "download-progress",
    listener: (progress: { percent: number }) => void,
  ): unknown;
  on(event: "error", listener: (error: Error) => void): unknown;
}

interface Options {
  enabled: boolean;
  automatic: boolean;
  createClient(): NativeUpdateClient;
  checkRelease(): Promise<Result<{ version: string }>>;
  confirmRestart(version: string): Promise<boolean>;
  installDownloaded(
    path: string,
    client: NativeUpdateClient,
  ): Promise<Result<void>>;
  openRelease(url: string): Promise<void>;
}

export function createNativeUpdateBackend(options: Options): UpdateBackend {
  let client: NativeUpdateClient | undefined;
  let downloaded: { version: string; path: string } | undefined;
  let installFailed = false;
  const listeners = new Set<(event: UpdateBackendEvent) => void>();
  const emit = (event: UpdateBackendEvent) =>
    listeners.forEach((listener) => listener(event));
  function getClient() {
    if (!client) {
      client = options.createClient();
      client.autoDownload = false;
      client.autoInstallOnAppQuit = false;
      client.allowPrerelease = false;
      client.allowDowngrade = false;
      client.disableDifferentialDownload = true;
      client.on("download-progress", ({ percent }) =>
        emit({ kind: "progress", percent }),
      );
      client.on("error", () => {
        installFailed = true;
        emit({ kind: "installFailed" });
      });
    }
    return client;
  }
  return {
    async check() {
      if (!options.enabled)
        return failure(
          "invalidInput",
          "Updates are available in installed builds of Muzo.",
        );
      const result = await options.checkRelease();
      return result.ok
        ? success({ ...result.value, automatic: options.automatic })
        : result;
    },
    async download(version) {
      if (!options.enabled || !options.automatic)
        return failure(
          "invalidInput",
          "Download this platform's installer from the release page.",
        );
      downloaded = undefined;
      try {
        const updater = getClient();
        const release = await updater.checkForUpdates();
        if (release?.updateInfo.version !== version)
          return failure(
            "remote",
            "The release changed. Check for updates again.",
          );
        const paths = await updater.downloadUpdate();
        if (paths.length !== 1 || !paths[0])
          return failure(
            "remote",
            "The update download did not produce a verified installer.",
          );
        downloaded = { version, path: paths[0] };
        return success(undefined);
      } catch {
        return failure(
          "remote",
          "Could not download and verify the update. Please retry.",
        );
      }
    },
    async install(version) {
      if (
        !options.enabled ||
        !options.automatic ||
        !client ||
        downloaded?.version !== version
      )
        return failure(
          "invalidInput",
          "Download and verify the update before installing it.",
        );
      try {
        if (!(await options.confirmRestart(version)))
          return success("cancelled");
        installFailed = false;
        const result = await options.installDownloaded(downloaded.path, client);
        if (!result.ok) return result;
        return installFailed
          ? failure(
              "storage",
              "Could not start the update installer. Please retry.",
            )
          : success("installing");
      } catch {
        return failure(
          "storage",
          "Could not start the update installer. Please retry.",
        );
      }
    },
    async openRelease(version) {
      if (!options.enabled || !/^\d+\.\d+\.\d+$/.test(version))
        return failure("invalidInput", "The release version is invalid.");
      try {
        await options.openRelease(`${MUZO_RELEASES_URL}/tag/v${version}`);
        return success(undefined);
      } catch {
        return failure("remote", "Could not open the release page.");
      }
    },
    subscribe(listener) {
      listeners.add(listener);
      return () => {
        listeners.delete(listener);
      };
    },
  };
}
