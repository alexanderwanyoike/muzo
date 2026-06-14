import type { LibraryDto } from "./types";

interface LibraryListProps {
  libraries: LibraryDto[];
}

const KIND_LABEL: Record<LibraryDto["kind"], string> = {
  filesystem: "Filesystem",
  dropbox: "Dropbox",
};

export default function LibraryList({ libraries }: LibraryListProps) {
  if (libraries.length === 0) {
    return (
      <p className="library-list__empty">
        No libraries yet. Add one above to get started.
      </p>
    );
  }

  return (
    <ul className="library-list">
      {libraries.map((library) => (
        <li key={library.id} className="library-list__row">
          <span className="library-list__name">{library.name}</span>
          <span className="library-list__kind">{KIND_LABEL[library.kind]}</span>
          <span className="library-list__location">{library.location}</span>
        </li>
      ))}
    </ul>
  );
}
