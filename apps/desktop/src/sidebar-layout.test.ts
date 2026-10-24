import { afterEach, expect, it } from "vitest";
import {
  loadSidebarWidth,
  saveSidebarWidth,
  clampSidebarWidth,
} from "./sidebar-layout";

afterEach(() => localStorage.clear());

it.each([
  [360, 800, 280],
  [100, 1280, 192],
  [500, 1280, 360],
  [Number.NaN, 1280, 240],
])(
  "keeps both panes usable when resizing from %s at %s pixels",
  (requested, viewport, expected) => {
    expect(clampSidebarWidth(requested, viewport)).toBe(expected);
  },
);

it("remembers the chosen sidebar width across new sessions", () => {
  expect(loadSidebarWidth()).toEqual({ ok: true, value: 240 });
  expect(saveSidebarWidth(320)).toEqual({ ok: true, value: undefined });
  expect(loadSidebarWidth()).toEqual({ ok: true, value: 320 });
});
