import { useState } from "react";
import type { PlayerWindowMode } from "../shared/player-window-mode";
import { failure, type Result } from "../shared/result";
import { getRuntimeBridge } from "./runtime-bridge";

async function setWindowMode(mode: PlayerWindowMode): Promise<Result<void>> {
  try {
    const runtime = getRuntimeBridge();
    if (!runtime.setPlayerWindowMode)
      return failure("remote", "Mini player requires the updated desktop app.");
    return await runtime.setPlayerWindowMode(mode);
  } catch {
    return failure(
      "remote",
      "The player window could not be changed. Try again.",
    );
  }
}

export function usePlayerWindowMode() {
  const [mode, setMode] = useState<PlayerWindowMode>("library");
  const [pending, setPending] = useState(false);
  const [error, setError] = useState<string | null>(null);
  async function changeMode(next: PlayerWindowMode) {
    if (pending) return;
    setPending(true);
    const result = await setWindowMode(next);
    setPending(false);
    setError(result.ok ? null : result.error.message);
    if (result.ok) setMode(next);
  }
  return { mode, pending, error, changeMode };
}
