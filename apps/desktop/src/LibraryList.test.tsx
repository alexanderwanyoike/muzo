import { describe, it, expect } from "vitest";
import { render, screen, within } from "@testing-library/react";
import LibraryList from "./LibraryList";
import type { LibraryDto } from "./types";

describe("LibraryList", () => {
  it("renders an empty state when there are no libraries", () => {
    render(<LibraryList libraries={[]} />);

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

    render(<LibraryList libraries={libraries} />);

    const firstRow = screen.getByText("First").closest("li")!;
    expect(within(firstRow).getByText("Filesystem")).toBeDefined();
    expect(within(firstRow).getByText("/home/user/A")).toBeDefined();

    const secondRow = screen.getByText("Second").closest("li")!;
    expect(within(secondRow).getByText("Dropbox")).toBeDefined();
    expect(within(secondRow).getByText("/dropbox/B")).toBeDefined();
  });
});
