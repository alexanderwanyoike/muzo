import { failure, success, type Result } from "./result";

export function newerStableVersion(
  next: string,
  current: string,
): Result<boolean> {
  const stable = /^(0|[1-9]\d*)\.(0|[1-9]\d*)\.(0|[1-9]\d*)$/;
  if (!stable.test(next) || !stable.test(current))
    return failure("remote", "The update did not identify a stable release.");
  const candidate = next.split(".").map(Number);
  const installed = current.split(".").map(Number);
  if (![...candidate, ...installed].every(Number.isSafeInteger))
    return failure("remote", "The update version is invalid.");
  for (const [index, part] of candidate.entries()) {
    if (part !== installed[index]) return success(part > installed[index]);
  }
  return success(false);
}
