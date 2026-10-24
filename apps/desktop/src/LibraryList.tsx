import { Cloud, Folder, X } from "lucide-react";
import type { ReactNode } from "react";
import type { LibraryDto } from "./types";

interface LibraryListProps {
  libraries: LibraryDto[];
  trackCounts: Record<string, number>;
  selectedLibraryId: string | null;
  onSelect: (libraryId: string | null) => void;
  renderTracks: (libraryId: string) => ReactNode;
  onRemove?: (library: LibraryDto) => void;
}

const KIND_LABEL: Record<LibraryDto["kind"], string> = {
  filesystem: "Filesystem",
  dropbox: "Dropbox",
};

export default function LibraryList({
  libraries,
  trackCounts,
  selectedLibraryId,
  onSelect,
  onRemove,
  renderTracks: TrackListSlot,
}: LibraryListProps) {
  if (libraries.length === 0) {
    return (
      <p className="library-list__empty">
        No libraries yet. Add one in Settings.
      </p>
    );
  }

  return (
    <ul className="library-list">
      {libraries.map((library) => {
        const count = trackCounts[library.id] ?? 0;
        const isSelected = selectedLibraryId === library.id;
        const trackContent = isSelected ? TrackListSlot(library.id) : null;
        return (
          <li
            key={library.id}
            className={
              isSelected
                ? "library-list__item library-list__item--selected"
                : "library-list__item"
            }
          >
            <div className="library-list__source">
            <button
              type="button"
              className="library-list__row"
              onClick={() => onSelect(isSelected ? null : library.id)}
              aria-label={`Show tracks for ${library.name}`}
              aria-pressed={isSelected}
              title={`${library.name}: ${library.location}`}
            >
              {library.kind === "dropbox" ? (
                <Cloud aria-hidden="true" />
              ) : (
                <Folder aria-hidden="true" />
              )}
              <span className="library-list__primary">
                <span className="library-list__name">{library.name}</span>
                <span className="library-list__kind">
                  {KIND_LABEL[library.kind]}
                </span>
                <span className="library-list__location">
                  {library.location}
                </span>
              </span>
              <span className="library-list__count">
                {count === 1 ? "1 track" : `${count} tracks`}
              </span>
            </button>
            {onRemove && <button type="button" className="library-list__remove" onClick={() => onRemove(library)} aria-label={`Remove ${library.name}`} title={`Remove ${library.name} from Muzo`}><X aria-hidden="true" /></button>}
            </div>

            {trackContent && (
              <div className="library-list__tracks">{trackContent}</div>
            )}
          </li>
        );
      })}
    </ul>
  );
}
