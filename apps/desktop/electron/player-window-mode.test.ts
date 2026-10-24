// @vitest-environment node
import { expect, it, vi } from "vitest";
import { EventEmitter } from "node:events";
import { createPlayerWindowModeController } from "./player-window-mode";

function windowFixture(maximized = false) {
  const bounds = { x: 40, y: 50, width: 1200, height: 800 };
  const events = new EventEmitter();
  let size = [1200, 800];
  return {
    events,
    on: events.on.bind(events),
    removeListener: events.removeListener.bind(events),
    getSize: () => size as [number, number],
    setShape: vi.fn(),
    isDestroyed: () => false,
    getNormalBounds: () => bounds,
    getMinimumSize: () => [800, 600] as [number, number],
    isMaximized: () => maximized,
    unmaximize: vi.fn(),
    maximize: vi.fn(),
    setMinimumSize: vi.fn(),
    setContentSize: vi.fn((width: number, height: number) => {
      size = [width, height];
    }),
    setBounds: vi.fn(),
  };
}

it("sizes compact and queue views and restores the original library window and maximized state", () => {
  const window = windowFixture(true);
  const controller = createPlayerWindowModeController(window);
  expect(controller.setMode("compact").ok).toBe(true);
  expect(window.unmaximize).toHaveBeenCalledOnce();
  expect(window.setMinimumSize).toHaveBeenLastCalledWith(360, 480);
  expect(window.setContentSize).toHaveBeenLastCalledWith(420, 560);
  expect(controller.setMode("compact-queue").ok).toBe(true);
  expect(window.setContentSize).toHaveBeenLastCalledWith(740, 560);
  expect(controller.setMode("library").ok).toBe(true);
  expect(window.setMinimumSize).toHaveBeenLastCalledWith(800, 600);
  expect(window.setBounds).toHaveBeenLastCalledWith({
    x: 40,
    y: 50,
    width: 1200,
    height: 800,
  });
  expect(window.maximize).toHaveBeenCalledOnce();
});

it("restores the previous window after a resize failure and allows a safe retry", () => {
  const window = windowFixture();
  const controller = createPlayerWindowModeController(window);
  window.setContentSize.mockImplementationOnce(() => {
    throw new Error("Window manager rejected resize");
  });
  expect(controller.setMode("compact").ok).toBe(false);
  expect(window.setMinimumSize).toHaveBeenLastCalledWith(800, 600);
  expect(window.setBounds).toHaveBeenLastCalledWith({
    x: 40,
    y: 50,
    width: 1200,
    height: 800,
  });
  expect(controller.setMode("compact").ok).toBe(true);
  expect(controller.setMode("library").ok).toBe(true);
  expect(window.setBounds).toHaveBeenLastCalledWith({
    x: 40,
    y: 50,
    width: 1200,
    height: 800,
  });
});

it("rounds native compact bounds, updates the shape after resize and clears it on library restore", () => {
  const window = windowFixture();
  const controller = createPlayerWindowModeController(window, "win32");
  expect(controller.setMode("compact").ok).toBe(true);
  const shape = window.setShape.mock.calls.at(-1)?.[0] as
    | { x: number; y: number; width: number; height: number }[]
    | undefined;
  const contains = (x: number, y: number) =>
    shape?.some(
      (rect) =>
        x >= rect.x &&
        x < rect.x + rect.width &&
        y >= rect.y &&
        y < rect.y + rect.height,
    );
  expect(contains(0, 0)).toBe(false);
  expect(contains(419, 559)).toBe(false);
  expect(contains(210, 0)).toBe(true);
  expect(contains(0, 280)).toBe(true);
  expect(contains(210, 559)).toBe(true);
  window.setContentSize(640, 480);
  window.events.emit("resize");
  const resized = window.setShape.mock.calls.at(-1)?.[0] as {
    x: number;
    y: number;
    width: number;
    height: number;
  }[];
  expect(Math.max(...resized.map((rect) => rect.x + rect.width))).toBe(640);
  expect(Math.max(...resized.map((rect) => rect.y + rect.height))).toBe(480);
  expect(controller.setMode("library").ok).toBe(true);
  expect(window.setShape).toHaveBeenLastCalledWith([]);
  const calls = window.setShape.mock.calls.length;
  window.events.emit("resize");
  expect(window.setShape).toHaveBeenCalledTimes(calls);
  window.events.emit("closed");
  expect(window.events.listenerCount("resize")).toBe(0);
});

it("rolls back a rejected shape and allows compact mode to be retried", () => {
  const window = windowFixture();
  const controller = createPlayerWindowModeController(window, "win32");
  window.setShape.mockImplementationOnce(() => {
    throw new Error("Native region rejected");
  });
  expect(controller.setMode("compact").ok).toBe(false);
  expect(window.setShape).toHaveBeenLastCalledWith([]);
  expect(window.setMinimumSize).toHaveBeenLastCalledWith(800, 600);
  expect(controller.setMode("compact").ok).toBe(true);
  expect(controller.setMode("library").ok).toBe(true);
});

it("keeps resize failures inside the native boundary and reports a failed rectangular fallback", () => {
  const window = windowFixture();
  const logger = { error: vi.fn() };
  const controller = createPlayerWindowModeController(window, "win32", logger);
  expect(controller.setMode("compact").ok).toBe(true);
  window.setShape.mockImplementation(() => {
    throw new Error("Native region unavailable");
  });
  expect(() => window.events.emit("resize")).not.toThrow();
  expect(window.setShape).toHaveBeenLastCalledWith([]);
  expect(logger.error).toHaveBeenCalledWith(expect.stringContaining("edges"));
});

it("preserves native compositor corners on macOS and native Wayland", () => {
  const mac = windowFixture();
  expect(
    createPlayerWindowModeController(mac, "darwin").setMode("compact").ok,
  ).toBe(true);
  expect(mac.setShape).not.toHaveBeenCalled();
  vi.stubEnv("XDG_SESSION_TYPE", "wayland");
  try {
    const wayland = windowFixture();
    expect(
      createPlayerWindowModeController(wayland, "linux").setMode("compact").ok,
    ).toBe(true);
    expect(wayland.setShape).not.toHaveBeenCalled();
  } finally {
    vi.unstubAllEnvs();
  }
});

it("keeps the X11 native title bar seam intact while rounding the bottom edges", () => {
  vi.stubEnv("XDG_SESSION_TYPE", "x11");
  try {
    const window = windowFixture();
    const controller = createPlayerWindowModeController(window, "linux");
    expect(controller.setMode("compact").ok).toBe(true);
    const shape = window.setShape.mock.calls.at(-1)?.[0] as {
      x: number;
      y: number;
      width: number;
      height: number;
    }[];
    expect(
      shape.some((rect) => rect.x === 0 && rect.y === 0 && rect.width === 420),
    ).toBe(true);
    expect(
      shape.some(
        (rect) => rect.x === 0 && rect.y <= 559 && rect.y + rect.height > 559,
      ),
    ).toBe(false);
  } finally {
    vi.unstubAllEnvs();
  }
});

it("keeps the X11 library region rectangular and synchronizes it after later resize", () => {
  vi.stubEnv("XDG_SESSION_TYPE", "x11");
  try {
    const window = windowFixture();
    const controller = createPlayerWindowModeController(window, "linux");
    expect(controller.setMode("compact").ok).toBe(true);
    expect(controller.setMode("library").ok).toBe(true);
    window.setContentSize(1100, 700);
    window.events.emit("resize");
    expect(window.setShape).toHaveBeenLastCalledWith([
      { x: 0, y: 0, width: 1100, height: 700 },
    ]);
  } finally {
    vi.unstubAllEnvs();
  }
});
