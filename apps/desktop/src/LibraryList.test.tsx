import { describe, it, expect, vi } from "vitest";
import { render, screen, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import LibraryList from "./LibraryList";
import type { LibraryDto } from "./types";

const NO_COUNTS: Record<string, number> = {};

it("filters sources by name or folder without changing the selected source", async () => {
  const user = userEvent.setup();
  const onSelect = vi.fn();
  render(
    <LibraryList
      libraries={[
        {
          id: "one",
          name: "Jazz",
          kind: "filesystem",
          location: "/music/local",
        },
        { id: "two", name: "Archive", kind: "dropbox", location: "/Jazz/live" },
        { id: "three", name: "Piano", kind: "filesystem", location: "/piano" },
      ]}
      trackCounts={NO_COUNTS}
      selectedLibraryId="three"
      onSelect={onSelect}
      renderTracks={() => null}
    />,
  );
  await user.type(
    screen.getByRole("searchbox", { name: "Search sources" }),
    "  JAZZ ",
  );
  expect(screen.getAllByRole("listitem")).toHaveLength(2);
  expect(screen.getByText("Jazz")).toBeDefined();
  expect(screen.getByText("Archive")).toBeDefined();
  expect(screen.queryByText("Piano")).toBeNull();
  expect(onSelect).not.toHaveBeenCalled();
});

describe("LibraryList", () => {
  it("does not present an unloaded source count as zero tracks", () => {
    render(
      <LibraryList
        libraries={[
          { id: "one", name: "Jazz", kind: "dropbox", location: "/jazz" },
        ]}
        trackCounts={NO_COUNTS}
        selectedLibraryId={null}
        onSelect={() => {}}
        renderTracks={() => null}
      />,
    );
    expect(screen.queryByText("0 tracks")).toBeNull();
  });

  it("clears a source search with Escape and keeps removal working after filtering", async () => {
    const user = userEvent.setup();
    const onRemove = vi.fn();
    const source: LibraryDto = {
      id: "one",
      name: "Jazz",
      kind: "dropbox",
      location: "/jazz",
    };
    render(
      <LibraryList
        libraries={[source]}
        trackCounts={NO_COUNTS}
        selectedLibraryId={null}
        onSelect={() => {}}
        onRemove={onRemove}
        renderTracks={() => null}
      />,
    );
    const search = screen.getByRole("searchbox", { name: "Search sources" });
    await user.type(search, "jazz{Escape}");
    expect((search as HTMLInputElement).value).toBe("");
    await user.type(search, "jazz");
    await user.click(screen.getByRole("button", { name: "Remove Jazz" }));
    expect(onRemove).toHaveBeenCalledWith(source);
  });

  it("clears an unmatched search and restores selection without selecting or removing anything", async () => {
    const user = userEvent.setup();
    const onSelect = vi.fn();
    const onRemove = vi.fn();
    render(
      <LibraryList
        libraries={[
          { id: "one", name: "Jazz", kind: "dropbox", location: "/jazz" },
        ]}
        trackCounts={NO_COUNTS}
        selectedLibraryId="one"
        onSelect={onSelect}
        onRemove={onRemove}
        renderTracks={() => null}
      />,
    );
    const search = screen.getByRole("searchbox", { name: "Search sources" });
    await user.type(search, "missing");
    expect(screen.getByText("No sources match your search.")).toBeDefined();
    expect(screen.getByText("0 of 1 sources")).toBeDefined();
    await user.click(
      screen.getByRole("button", { name: "Clear source search" }),
    );
    expect(search).toBe(document.activeElement);
    expect(
      screen
        .getByRole("button", { name: "Show tracks for Jazz" })
        .getAttribute("aria-pressed"),
    ).toBe("true");
    expect(onSelect).not.toHaveBeenCalled();
    expect(onRemove).not.toHaveBeenCalled();
  });

  it("renders an empty state when there are no libraries", () => {
    render(
      <LibraryList
        libraries={[]}
        trackCounts={NO_COUNTS}
        selectedLibraryId={null}
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
        selectedLibraryId={null}
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
        selectedLibraryId={null}
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
        selectedLibraryId={null}
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
        selectedLibraryId="lib-1"
        onSelect={() => {}}
        renderTracks={(id) => <p>Tracks for {id}</p>}
      />,
    );

    expect(screen.getByText("Tracks for lib-1")).toBeDefined();
  });
});

it("selects a source by clicking its name", async () => {
  const onSelect = vi.fn();
  render(
    <LibraryList
      libraries={[
        {
          id: "source",
          name: "Music folder",
          kind: "filesystem",
          location: "/music",
        },
      ]}
      trackCounts={{}}
      selectedLibraryId={null}
      onSelect={onSelect}
      renderTracks={() => null}
    />,
  );
  await userEvent.click(screen.getByText("Music folder"));
  expect(onSelect).toHaveBeenCalledWith("source");
});
