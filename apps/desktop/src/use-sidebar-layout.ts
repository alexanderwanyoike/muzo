import { useEffect, useState } from "react";
import {
  clampSidebarWidth,
  defaultSidebarWidth,
  loadSidebarWidth,
  minimumSidebarWidth,
  saveSidebarWidth,
  sidebarWidthLimit,
} from "./sidebar-layout";

export function useSidebarLayout() {
  const [initial] = useState(loadSidebarWidth);
  const [preferred, setPreferred] = useState(
    initial.ok ? initial.value : defaultSidebarWidth,
  );
  const [viewport, setViewport] = useState(window.innerWidth);
  const [draft, setDraft] = useState<number | null>(null);
  const [error, setError] = useState<string | null>(
    initial.ok ? null : initial.error.message,
  );
  useEffect(() => {
    const resize = () => setViewport(window.innerWidth);
    window.addEventListener("resize", resize);
    return () => window.removeEventListener("resize", resize);
  }, []);
  const width = clampSidebarWidth(draft ?? preferred, viewport);
  function commit(requested: number) {
    const next = clampSidebarWidth(requested, viewport);
    setDraft(null);
    setPreferred(next);
    const saved = saveSidebarWidth(next);
    setError(saved.ok ? null : saved.error.message);
  }
  return {
    width,
    minimum: minimumSidebarWidth,
    maximum: sidebarWidthLimit(viewport),
    preview: setDraft,
    cancel: () => setDraft(null),
    commit,
    error,
  };
}
