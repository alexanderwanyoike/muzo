// @vitest-environment node
import { expect, it, vi } from "vitest";
import { installPlayerWindowMode } from "./install-player-window-mode";

it("accepts only a known player window's main frame and validates requested modes", () => {
  const handle = vi.fn();
  const frame = {};
  const event = { senderFrame: frame, sender: { mainFrame: frame } };
  const window = {
    isDestroyed: () => false,
    getNormalBounds: () => ({ x: 20, y: 30, width: 1200, height: 800 }),
    getMinimumSize: () => [800, 600] as [number, number],
    isMaximized: () => false,
    unmaximize: vi.fn(),
    maximize: vi.fn(),
    setMinimumSize: vi.fn(),
    setContentSize: vi.fn(),
    setBounds: vi.fn(),
  };
  const findWindow = vi.fn().mockReturnValue(window);
  installPlayerWindowMode({ handle }, findWindow);
  expect(handle).toHaveBeenCalledWith(
    "muzo:player-window-mode",
    expect.any(Function),
  );
  const command = handle.mock.calls[0][1] as (
    event: unknown,
    mode: unknown,
  ) => { ok: boolean };
  expect(command({ ...event, senderFrame: {} }, "compact").ok).toBe(false);
  expect(findWindow).not.toHaveBeenCalled();
  expect(command(event, "transparent").ok).toBe(false);
  expect(window.setContentSize).not.toHaveBeenCalled();
  expect(command(event, "compact").ok).toBe(true);
  expect(window.setContentSize).toHaveBeenCalledWith(420, 560);
  expect(command(event, "compact-queue").ok).toBe(true);
  expect(command(event, "library").ok).toBe(true);
  expect(window.setBounds).toHaveBeenCalledWith({
    x: 20,
    y: 30,
    width: 1200,
    height: 800,
  });
  findWindow.mockReturnValue(null);
  expect(command(event, "compact").ok).toBe(false);
});
