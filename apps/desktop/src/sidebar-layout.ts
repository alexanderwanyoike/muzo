import { failure, type Result } from "../shared/result";

export const minimumSidebarWidth = 192;
export const maximumSidebarWidth = 360;
export const defaultSidebarWidth = 240;
const storageKey = "muzo.sidebar-width.v1";

function validWidth(value: unknown): value is number {
  return (
    typeof value === "number" &&
    Number.isFinite(value) &&
    value >= minimumSidebarWidth &&
    value <= maximumSidebarWidth
  );
}

export function loadSidebarWidth(): Result<number> {
  try {
    const stored = localStorage.getItem(storageKey);
    if (stored === null) return { ok: true, value: defaultSidebarWidth };
    const width: unknown = JSON.parse(stored);
    return validWidth(width)
      ? { ok: true, value: width }
      : failure(
          "storage",
          "Saved sidebar width could not be read. Using the default.",
        );
  } catch {
    return failure(
      "storage",
      "Sidebar width could not be loaded. Using the default.",
    );
  }
}

export function saveSidebarWidth(width: number): Result<void> {
  if (!validWidth(width))
    return failure("invalidInput", "Sidebar width is out of range.");
  try {
    localStorage.setItem(storageKey, JSON.stringify(width));
    return { ok: true, value: undefined };
  } catch {
    return failure(
      "storage",
      "Sidebar width changed for this session, but could not be saved.",
    );
  }
}

const minimumContentWidth = 520;
export function sidebarWidthLimit(viewportWidth: number): number {
  return Math.max(
    minimumSidebarWidth,
    Math.min(maximumSidebarWidth, viewportWidth - minimumContentWidth),
  );
}
export function clampSidebarWidth(
  width: number,
  viewportWidth: number,
): number {
  const requested = Number.isFinite(width)
    ? Math.round(width)
    : defaultSidebarWidth;
  return Math.max(
    minimumSidebarWidth,
    Math.min(sidebarWidthLimit(viewportWidth), requested),
  );
}
