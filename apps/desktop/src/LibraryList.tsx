import type { LibraryDto } from "./types";

interface LibraryListProps {
  libraries: LibraryDto[];
  trackCounts: Record<string, number>;
  scanningLibraryId: string | null;
  onScan: (libraryId: string) => void;
}

const KIND_LABEL: Record<LibraryDto["kind"], string> = {
  filesystem: "Filesystem",
  dropbox: "Dropbox",
};

export default function LibraryList({
  libraries,
  trackCounts,
  scanningLibraryId,
  onScan,
}: LibraryListProps) {
  if (libraries.length === 0) {
    return (
      <p className="library-list__empty">
        No libraries yet. Add one above to get started.
      </p>
    );
  }

  return (
    <ul className="library-list">
      {libraries.map((library) => {
        const count = trackCounts[library.id] ?? 0;
        const isScanning = scanningLibraryId === library.id;
        return (
          <li key={library.id} className="library-list__row">
            <div className="library-list__primary">
              <span className="library-list__name">{library.name}</span>
              <span className="library-list__kind">{KIND_LABEL[library.kind]}</span>
            </div>
            <span className="library-list__location">{library.location}</span>
            <div className="library-list__actions">
              <span className="library-list__count">
                {count === 1 ? "1 track" : `${count} tracks`}
              </span>
              <button
                type="button"
                disabled={isScanning}
                onClick={() => onScan(library.id)}
              >
                {isScanning ? "Scanning..." : "Scan"}
              </button>
            </div>
          </li>
        );
      })}
    </ul>
  );
}
