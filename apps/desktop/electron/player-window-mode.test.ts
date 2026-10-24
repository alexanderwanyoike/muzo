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

it("keeps compact and queue windows rectangular through resize and library restore", () => {
  const window = windowFixture();
  window.setShape.mockImplementation(() => { throw new Error("Native shaping should not be needed"); });
  const controller = createPlayerWindowModeController(window);
  expect(controller.setMode("compact").ok).toBe(true);
  window.setContentSize(360, 480);
  window.events.emit("resize");
  expect(controller.setMode("compact-queue").ok).toBe(true);
  window.setContentSize(640, 480);
  window.events.emit("resize");
  expect(controller.setMode("library").ok).toBe(true);
  expect(window.setShape).not.toHaveBeenCalled();
  expect(window.events.listenerCount("resize")).toBe(0);
});
