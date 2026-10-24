import { afterEach, expect, it } from "vitest";
import { loadSidebarWidth, saveSidebarWidth } from "./sidebar-layout";

afterEach(() => localStorage.clear());

it("remembers the chosen sidebar width across new sessions", () => {
  expect(loadSidebarWidth()).toEqual({ ok: true, value: 240 });
  expect(saveSidebarWidth(320)).toEqual({ ok: true, value: undefined });
  expect(loadSidebarWidth()).toEqual({ ok: true, value: 320 });
});
