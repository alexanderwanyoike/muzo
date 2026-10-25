import { failure, type Result } from "../shared/result";

export const MATERIALS = [
  { id: "studio", name: "Studio", detail: "Quiet, clear and contemporary" },
  {
    id: "aluminium",
    name: "Brushed aluminium",
    detail: "Fine matte grain and satin highlights",
  },
  {
    id: "paper",
    name: "Warm paper",
    detail: "Soft neutrals with a little warmth",
  },
  {
    id: "carbon",
    name: "Carbon fibre",
    detail: "Fine weave and cool graphite",
  },
  { id: "walnut", name: "Walnut", detail: "Soft wood grain and warm tones" },
  {
    id: "porcelain",
    name: "Porcelain",
    detail: "Smooth ceramic and gentle highlights",
  },
  {
    id: "glass",
    name: "Frosted glass",
    detail: "Soft translucency and blurred colour",
  },
] as const;
export const MODES = ["light", "dark"] as const;
const classicAccents = [
  { id: "cobalt", name: "Cobalt" },
  { id: "copper", name: "Copper" },
  { id: "sage", name: "Sage" },
  { id: "plum", name: "Plum" },
  { id: "amber", name: "Amber" },
  { id: "teal", name: "Teal" },
  { id: "rose", name: "Rose" },
  { id: "slate", name: "Slate" },
  { id: "violet", name: "Violet" },
  { id: "ice", name: "Ice" },
] as const;
const glowAccents = [
  { id: "glow-green", name: "Glow green" },
  { id: "glow-cyan", name: "Glow cyan" },
  { id: "glow-blue", name: "Glow blue" },
  { id: "glow-violet", name: "Glow violet" },
  { id: "glow-pink", name: "Glow pink" },
  { id: "glow-orange", name: "Glow orange" },
] as const;
export const ACCENTS = [...classicAccents, ...glowAccents] as const;
export const ACCENT_GROUPS = [
  { name: "Classic", accents: classicAccents },
  { name: "Glow", accents: glowAccents },
] as const;
export interface Appearance {
  theme: (typeof MATERIALS)[number]["id"];
  mode: (typeof MODES)[number];
  accent: (typeof ACCENTS)[number]["id"];
}
const STORAGE_KEY = "muzo.appearance.v1";
export const DEFAULT_APPEARANCE: Appearance = {
  theme: "studio",
  mode: "light",
  accent: "cobalt",
};

function isAppearance(value: unknown): value is Appearance {
  return (
    typeof value === "object" &&
    value !== null &&
    "theme" in value &&
    MATERIALS.some(({ id }) => id === value.theme) &&
    "mode" in value &&
    MODES.some((mode) => mode === value.mode) &&
    "accent" in value &&
    ACCENTS.some(({ id }) => id === value.accent)
  );
}
export function loadAppearance(): Result<Appearance> {
  try {
    const stored = localStorage.getItem(STORAGE_KEY);
    if (stored === null) return { ok: true, value: DEFAULT_APPEARANCE };
    const parsed: unknown = JSON.parse(stored);
    return isAppearance(parsed)
      ? { ok: true, value: parsed }
      : failure(
          "storage",
          "Saved appearance could not be read. Choose your preferences again.",
        );
  } catch {
    return failure(
      "storage",
      "Appearance could not be loaded. Changes will apply to this session.",
    );
  }
}
export function saveAppearance(appearance: Appearance): Result<void> {
  try {
    localStorage.setItem(STORAGE_KEY, JSON.stringify(appearance));
    return { ok: true, value: undefined };
  } catch {
    return failure(
      "storage",
      "Appearance changed for this session, but could not be saved.",
    );
  }
}
export function applyAppearance(appearance: Appearance): void {
  const root = document.documentElement;
  root.dataset.theme = appearance.theme;
  root.dataset.mode = appearance.mode;
  root.dataset.accent = appearance.accent;
  root.style.colorScheme = appearance.mode;
}
