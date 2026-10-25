import { spawn } from "node:child_process";
import { dirname } from "node:path";
import { failure, success, type Result } from "../../application/result";

export function launchAppImage(
  file: string,
  args: string[],
): Promise<Result<void>> {
  const failed = () =>
    failure(
      "storage",
      `The update is installed, but Muzo could not restart. Open ${file} to continue.`,
    );
  return new Promise((resolve) => {
    try {
      const child = spawn(file, args, {
        cwd: dirname(file),
        detached: true,
        stdio: "ignore",
      });
      child.once("error", () => resolve(failed()));
      child.once("spawn", () => {
        child.unref();
        resolve(success(undefined));
      });
    } catch {
      resolve(failed());
    }
  });
}
