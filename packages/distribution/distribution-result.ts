export type DistributionResult<T> =
  | { ok: true; value: T }
  | {
      ok: false;
      error: { kind: "invalidRelease" | "storage"; message: string };
    };

export function distributionFailure(
  kind: "invalidRelease" | "storage",
  message: string,
): DistributionResult<never> {
  return { ok: false, error: { kind, message } };
}
