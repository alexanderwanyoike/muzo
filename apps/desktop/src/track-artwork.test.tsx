import { afterEach, expect, it, vi } from "vitest";
import { act, cleanup, render, screen } from "@testing-library/react";
import TrackArtwork from "./track-artwork";
import { installTestRuntime, removeTestRuntime } from "./test-runtime";

afterEach(() => {
  cleanup();
  vi.unstubAllGlobals();
  removeTestRuntime();
});

it("loads embedded artwork when its track enters the visible listening area", async () => {
  let reveal: (() => void) | undefined;
  const disconnect = vi.fn();
  vi.stubGlobal(
    "IntersectionObserver",
    vi.fn((callback: IntersectionObserverCallback) => {
      reveal = () =>
        callback(
          [{ isIntersecting: true }] as IntersectionObserverEntry[],
          { disconnect } as unknown as IntersectionObserver,
        );
      return { observe: vi.fn(), disconnect };
    }),
  );
  const invoke = vi
    .fn()
    .mockResolvedValue({ ok: true, value: "data:image/png;base64,cover" });
  installTestRuntime({ invoke, openDirectory: vi.fn() });
  render(
    <TrackArtwork
      track={{
        id: "track",
        libraryId: "music",
        title: "Song",
        album: "Evening",
      }}
    />,
  );
  expect(invoke).not.toHaveBeenCalled();
  act(() => reveal?.());
  const image = await screen.findByRole("img", { name: "Artwork for Evening" });
  expect(image.getAttribute("src")).toBe("data:image/png;base64,cover");
  expect(invoke).toHaveBeenCalledWith("get_track_artwork", {
    input: { libraryId: "music", trackId: "track" },
  });
});

it("reloads the cover when its library refresh completes", async () => {
  const invoke = vi.fn().mockResolvedValueOnce({ ok: true, value: "data:image/png;base64,old" }).mockResolvedValueOnce({ ok: true, value: "data:image/png;base64,new" });
  installTestRuntime({ invoke, openDirectory: vi.fn() });
  const track = { id: "track", libraryId: "music", title: "Song", album: "Evening" };
  const view = render(<TrackArtwork track={track} eager refreshKey={0} />);
  await screen.findByRole("img", { name: "Artwork for Evening" });
  view.rerender(<TrackArtwork track={track} eager refreshKey={1} />);
  await vi.waitFor(() => expect(screen.getByRole("img", { name: "Artwork for Evening" }).getAttribute("src")).toBe("data:image/png;base64,new"));
});
