import type { Result } from "../result";

export type UpdateBackendEvent =
  | { kind: "progress"; percent: number }
  | { kind: "installFailed" };
export interface UpdateBackend {
  check(): Promise<Result<{ version: string; automatic: boolean }>>;
  download(version: string): Promise<Result<void>>;
  install(version: string): Promise<Result<"installing" | "cancelled">>;
  openRelease(version: string): Promise<Result<void>>;
  subscribe(listener: (event: UpdateBackendEvent) => void): () => void;
}
