import SidebarPanel from "./sidebar-panel";
import { FolderTree, Plus } from "lucide-react";
import LibraryList, { type LibraryListProps } from "./LibraryList";
import type { LibraryDto } from "./types";

type Props = Omit<LibraryListProps, "libraries" | "renderTracks"> & {
  libraries: LibraryDto[] | null;
  onAddSource: () => void;
};

export default function SourcesPanel({
  libraries,
  onAddSource,
  ...listProps
}: Props) {
  return (
    <SidebarPanel title="Sources" headingId="sources-panel-heading" icon={<FolderTree aria-hidden="true" />} count={libraries?.length ?? null} footer={
      <button type="button" onClick={onAddSource}><Plus aria-hidden="true" /> Add source</button>
    }>
      {libraries === null ? (
        <p className="app__placeholder">Loading sources...</p>
      ) : (
        <LibraryList
          {...listProps}
          libraries={libraries}
          renderTracks={() => null}
        />
      )}
    </SidebarPanel>
  );
}
