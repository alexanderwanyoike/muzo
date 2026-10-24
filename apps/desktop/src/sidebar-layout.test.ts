import { afterEach, expect, it, vi } from "vitest";
import {
  loadSidebarWidth,
  saveSidebarWidth,
  clampSidebarWidth,
} from "./sidebar-layout";

afterEach(() => { localStorage.clear(); vi.restoreAllMocks(); });

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


it.each(["broken", "null", "[]", "9999", "false"])("rejects an invalid stored width %s", (stored) => {
  vi.spyOn(Storage.prototype, "getItem").mockReturnValue(stored);
  expect(loadSidebarWidth()).toMatchObject({ ok: false, error: { kind: "storage" } });
});

it("reports unavailable preference storage without throwing", () => {
  vi.spyOn(Storage.prototype, "getItem").mockImplementation(() => { throw new Error("Unavailable"); });
  vi.spyOn(Storage.prototype, "setItem").mockImplementation(() => { throw new Error("Unavailable"); });
  expect(loadSidebarWidth()).toMatchObject({ ok: false, error: { kind: "storage" } });
  expect(saveSidebarWidth(260)).toMatchObject({ ok: false, error: { kind: "storage" } });
  expect(saveSidebarWidth(Number.NaN)).toMatchObject({ ok: false, error: { kind: "invalidInput" } });
});
