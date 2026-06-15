import { describe, it, expect, vi } from "vitest";
import { render, screen, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import LibraryList from "./LibraryList";
import type { LibraryDto } from "./types";

const NO_COUNTS: Record<string, number> = {};

describe("LibraryList", () => {
  it("renders an empty state when there are no libraries", () => {
    render(
      <LibraryList
        libraries={[]}
        trackCounts={NO_COUNTS}
        scanningLibraryId={null}
        selectedLibraryId={null}
        onScan={() => {}}
        onSelect={() => {}}
        renderTracks={() => null}
      />,
    );

    expect(screen.getByText(/no libraries yet/i)).toBeDefined();
  });

  it("renders one row per library with name, kind, and location", () => {
    const libraries: LibraryDto[] = [
      {
        id: "lib-1",
        name: "First",
        kind: "filesystem",
        location: "/home/user/A",
      },
      {
        id: "lib-2",
        name: "Second",
        kind: "dropbox",
        location: "/dropbox/B",
      },
    ];

    render(
      <LibraryList
        libraries={libraries}
        trackCounts={NO_COUNTS}
        scanningLibraryId={null}
        selectedLibraryId={null}
        onScan={() => {}}
        onSelect={() => {}}
        renderTracks={() => null}
      />,
    );

    const firstRow = screen.getByText("First").closest("li")!;
    expect(within(firstRow).getByText("Filesystem")).toBeDefined();
    expect(within(firstRow).getByText("/home/user/A")).toBeDefined();

    const secondRow = screen.getByText("Second").closest("li")!;
    expect(within(secondRow).getByText("Dropbox")).toBeDefined();
    expect(within(secondRow).getByText("/dropbox/B")).toBeDefined();
  });

  it("shows the track count for each library from the trackCounts map", () => {
    const libraries: LibraryDto[] = [
      { id: "lib-1", name: "One", kind: "filesystem", location: "/a" },
      { id: "lib-2", name: "Two", kind: "filesystem", location: "/b" },
    ];

    render(
      <LibraryList
        libraries={libraries}
        trackCounts={{ "lib-1": 1, "lib-2": 42 }}
        scanningLibraryId={null}
        selectedLibraryId={null}
        onScan={() => {}}
        onSelect={() => {}}
        renderTracks={() => null}
      />,
    );

    expect(screen.getByText("1 track")).toBeDefined();
    expect(screen.getByText("42 tracks")).toBeDefined();
  });

  it("triggers onSelect with the library id when expand is clicked", async () => {
    const user = userEvent.setup();
    const onSelect = vi.fn();
    const libraries: LibraryDto[] = [
      { id: "lib-1", name: "One", kind: "filesystem", location: "/a" },
    ];

    render(
      <LibraryList
        libraries={libraries}
        trackCounts={NO_COUNTS}
        scanningLibraryId={null}
        selectedLibraryId={null}
        onScan={() => {}}
        onSelect={onSelect}
        renderTracks={() => null}
      />,
    );

    await user.click(screen.getByRole("button", { name: /show tracks/i }));
    expect(onSelect).toHaveBeenCalledWith("lib-1");
  });

  it("renders track content for the selected library via renderTracks", () => {
    const libraries: LibraryDto[] = [
      { id: "lib-1", name: "One", kind: "filesystem", location: "/a" },
    ];

    render(
      <LibraryList
        libraries={libraries}
        trackCounts={NO_COUNTS}
        scanningLibraryId={null}
        selectedLibraryId="lib-1"
        onScan={() => {}}
        onSelect={() => {}}
        renderTracks={(id) => <p>Tracks for {id}</p>}
      />,
    );

    expect(screen.getByText("Tracks for lib-1")).toBeDefined();
  });
});
