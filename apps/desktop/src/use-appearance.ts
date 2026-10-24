import { useEffect, useState } from "react";
import {
  applyAppearance,
  DEFAULT_APPEARANCE,
  loadAppearance,
  saveAppearance,
  type Appearance,
} from "./appearance";

export function useAppearance() {
  const [initial] = useState(loadAppearance);
  const [appearance, setAppearance] = useState(
    initial.ok ? initial.value : DEFAULT_APPEARANCE,
  );
  const [error, setError] = useState<string | null>(
    initial.ok ? null : initial.error.message,
  );
  useEffect(() => {
    applyAppearance(appearance);
  }, [appearance]);
  function changeAppearance(next: Appearance) {
    setAppearance(next);
    const saved = saveAppearance(next);
    setError(saved.ok ? null : saved.error.message);
  }
  return { appearance, changeAppearance, error };
}
