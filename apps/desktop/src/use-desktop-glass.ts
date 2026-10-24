import { useEffect, useState } from "react";
import { failure, type Result } from "../shared/result";
import { getRuntimeBridge } from "./runtime-bridge";

async function requestGlass(enabled: boolean): Promise<Result<boolean>> {
  try {
    const runtime = getRuntimeBridge();
    if (!runtime.setDesktopGlass) return { ok: true, value: false };
    return await runtime.setDesktopGlass(enabled);
  } catch {
    return failure("remote", "Desktop transparency could not be updated.");
  }
}

export function useDesktopGlass(enabled: boolean) {
  const [error, setError] = useState<string | null>(null);
  useEffect(() => {
    let cancelled = false;
    void requestGlass(enabled).then((result) => {
      if (cancelled) return;
      const root = document.documentElement;
      if (result.ok && result.value && enabled)
        root.dataset.desktopGlass = "true";
      else delete root.dataset.desktopGlass;
      setError(result.ok ? null : result.error.message);
    });
    return () => {
      cancelled = true;
      delete document.documentElement.dataset.desktopGlass;
    };
  }, [enabled]);
  return error;
}
