import { Cloud, Folder, Search, X } from "lucide-react";
import { useRef, useState, type ReactNode } from "react";
import type { LibraryDto } from "./types";

export interface LibraryListProps {
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
  const [query, setQuery] = useState("");
  const searchInput = useRef<HTMLInputElement>(null);
  const clearSearch = () => {
    setQuery("");
    searchInput.current?.focus();
  };
  const search = query.trim().toLowerCase();
  const visibleLibraries = libraries.filter((library) =>
    [library.name, library.location].some((value) =>
      value.toLowerCase().includes(search),
    ),
  );
  if (libraries.length === 0) {
    return (
      <p className="library-list__empty">
        No libraries yet. Add one in Settings.
      </p>
    );
  }

  return (
    <div className="source-browser">
      <div className="source-browser__search">
        <Search aria-hidden="true" />
        <input
          ref={searchInput}
          type="search"
          aria-label="Search sources"
          placeholder="Search sources"
          value={query}
          onChange={(event) => setQuery(event.target.value)}
          onKeyDown={(event) => {
            if (event.key === "Escape") clearSearch();
          }}
        />
        {query && (
          <button
            type="button"
            aria-label="Clear source search"
            onClick={clearSearch}
          >
            <X aria-hidden="true" />
          </button>
        )}
      </div>
      <p className="source-browser__count" aria-live="polite">
        {visibleLibraries.length} of {libraries.length} sources
      </p>
      {visibleLibraries.length === 0 && (
        <p className="library-list__empty">No sources match your search.</p>
      )}
      <ul className="library-list" aria-label="Sources">
        {visibleLibraries.map((library) => {
          const count = trackCounts[library.id];
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
                  {count !== undefined && (
                    <span className="library-list__count">
                      {count === 1 ? "1 track" : `${count} tracks`}
                    </span>
                  )}
                </button>
                {onRemove && (
                  <button
                    type="button"
                    className="library-list__remove"
                    onClick={() => onRemove(library)}
                    aria-label={`Remove ${library.name}`}
                    title={`Remove ${library.name} from Muzo`}
                  >
                    <X aria-hidden="true" />
                  </button>
                )}
              </div>

              {trackContent && (
                <div className="library-list__tracks">{trackContent}</div>
              )}
            </li>
          );
        })}
      </ul>
    </div>
  );
}
