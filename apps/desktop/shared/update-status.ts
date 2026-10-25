export type UpdateStatus =
  | { state: "idle" | "checking" | "current" }
  | { state: "available"; version: string; automatic: boolean }
  | { state: "downloading"; version: string; progress: number }
  | { state: "ready" | "installing"; version: string }
  | { state: "failed"; message: string };
