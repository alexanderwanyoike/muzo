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
    <section className="sources-panel" aria-labelledby="sources-panel-heading">
      <header className="sources-panel__header">
        <FolderTree aria-hidden="true" />
        <h2 id="sources-panel-heading">Sources</h2>
        <span className="sources-panel__total" aria-live="polite">
          {libraries?.length ?? "..."}
        </span>
      </header>
      {libraries === null ? (
        <p className="app__placeholder">Loading sources...</p>
      ) : (
        <LibraryList
          {...listProps}
          libraries={libraries}
          renderTracks={() => null}
        />
      )}
      <footer className="sources-panel__footer">
        <button type="button" onClick={onAddSource}>
          <Plus aria-hidden="true" /> Add source
        </button>
      </footer>
    </section>
  );
}
