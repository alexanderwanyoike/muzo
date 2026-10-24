export interface AppFailure {
  kind: "invalidInput" | "notConnected" | "authorization" | "storage" | "remote";
  message: string;
}

export type Result<T> = { ok: true; value: T } | { ok: false; error: AppFailure };

export function failure(kind: AppFailure["kind"], message: string): Result<never> {
  return { ok: false, error: { kind, message } };
}
