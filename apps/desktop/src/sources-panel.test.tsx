import { expect, it, vi } from "vitest";
import { render, screen, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import SourcesPanel from "./sources-panel";

it("groups searchable sources and the add action in a named panel", async () => {
  const onAddSource = vi.fn();
  render(
    <SourcesPanel
      libraries={[
        { id: "one", name: "Jazz", kind: "filesystem", location: "/Jazz" },
      ]}
      trackCounts={{}}
      selectedLibraryId={null}
      onSelect={() => {}}
      onAddSource={onAddSource}
    />,
  );
  const panel = screen.getByRole("region", { name: "Sources" });
  expect(
    within(panel).getByRole("searchbox", { name: "Search sources" }),
  ).toBeDefined();
  expect(
    within(panel).getByRole("button", { name: "Show tracks for Jazz" }),
  ).toBeDefined();
  await userEvent.click(
    within(panel).getByRole("button", { name: "Add source" }),
  );
  expect(onAddSource).toHaveBeenCalledOnce();
});
