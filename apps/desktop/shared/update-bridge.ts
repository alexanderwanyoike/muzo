import type { Result } from "./result";
import type { UpdateStatus } from "./update-status";

export interface UpdateBridge {
  getStatus(): Promise<Result<UpdateStatus>>;
  command(command: "check" | "download" | "install"): Promise<Result<void>>;
  subscribe(listener: (status: UpdateStatus) => void): () => void;
}
