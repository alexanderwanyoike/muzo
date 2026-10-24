// @vitest-environment node

import { beforeEach, describe, expect, it, vi } from "vitest";
import { watch, type FSWatcher } from "chokidar";

import { NodeFilesystemLibraryWatcher } from "./node-filesystem-library-watcher";

vi.mock("chokidar", () => ({
  watch: vi.fn(),
}));

describe("NodeFilesystemLibraryWatcher", () => {
  beforeEach(() => {
    vi.useFakeTimers();
    vi.mocked(watch).mockReset();
  });

  it("watches a library root and debounces file change bursts", async () => {
    const watcher = new FakeChokidarWatcher();
    vi.mocked(watch).mockReturnValue(watcher.asFsWatcher());
    const onChange = vi.fn();

    await new NodeFilesystemLibraryWatcher().watch("lib-1", "/music", onChange);

    expect(watch).toHaveBeenCalledWith("/music", {
      ignoreInitial: true,
      persistent: false,
    });

    watcher.emit("all");
    watcher.emit("all");
    await vi.advanceTimersByTimeAsync(499);
    expect(onChange).not.toHaveBeenCalled();

    await vi.advanceTimersByTimeAsync(1);
    expect(onChange).toHaveBeenCalledTimes(1);
    expect(onChange).toHaveBeenCalledWith("lib-1");
  });

  it("does not register the same library twice", async () => {
    vi.mocked(watch).mockReturnValue(new FakeChokidarWatcher().asFsWatcher());
    const filesystemWatcher = new NodeFilesystemLibraryWatcher();

    await filesystemWatcher.watch("lib-1", "/music", vi.fn());
    await filesystemWatcher.watch("lib-1", "/music", vi.fn());

    expect(watch).toHaveBeenCalledTimes(1);
  });
  it("closes a removed source watcher and cancels its pending change", async () => {
    const watcher = new FakeChokidarWatcher();
    const close = vi.fn().mockResolvedValue(undefined);
    vi.mocked(watch).mockReturnValue(
      Object.assign(watcher.asFsWatcher(), { close }),
    );
    const filesystemWatcher = new NodeFilesystemLibraryWatcher();
    const onChange = vi.fn();
    await filesystemWatcher.watch("source", "/music", onChange);
    watcher.emit("all");
    await filesystemWatcher.unwatch("source");
    await vi.advanceTimersByTimeAsync(1000);
    expect(onChange).not.toHaveBeenCalled();
    expect(close).toHaveBeenCalledOnce();
    await filesystemWatcher.watch("source", "/music", onChange);
    expect(watch).toHaveBeenCalledTimes(2);
  });
  it("ignores events delivered while a watcher is closing", async () => {
    const watcher = new FakeChokidarWatcher();
    let finish: (() => void) | undefined;
    const close = vi.fn(() => new Promise<void>(resolve => { finish = resolve; }));
    vi.mocked(watch).mockReturnValue(Object.assign(watcher.asFsWatcher(), { close }));
    const filesystemWatcher = new NodeFilesystemLibraryWatcher();
    const onChange = vi.fn();
    await filesystemWatcher.watch("source", "/music", onChange);
    const closing = filesystemWatcher.unwatch("source");
    watcher.emit("all");
    finish?.();
    await closing;
    await vi.advanceTimersByTimeAsync(1000);
    expect(onChange).not.toHaveBeenCalled();
  });

});

class FakeChokidarWatcher {
  private readonly handlers = new Map<string, (() => void)[]>();

  on(eventName: string, handler: () => void): this {
    const handlers = this.handlers.get(eventName) ?? [];
    handlers.push(handler);
    this.handlers.set(eventName, handlers);
    return this;
  }

  emit(eventName: string): void {
    for (const handler of this.handlers.get(eventName) ?? []) {
      handler();
    }
  }

  asFsWatcher(): FSWatcher {
    return this as unknown as FSWatcher;
  }
}
