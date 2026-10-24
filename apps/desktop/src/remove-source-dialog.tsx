import { useEffect, useRef, useState } from "react";
import type { LibraryDto } from "./types";
import { removeLibrary } from "./remove-source-api";
import "./remove-source-dialog.css";

interface Props {
  library: LibraryDto;
  onRemoved: (libraryId: string) => void;
  onClose: () => void;
}
export default function RemoveSourceDialog({
  library,
  onRemoved,
  onClose,
}: Props) {
  const dialog = useRef<HTMLDialogElement>(null);
  const cancel = useRef<HTMLButtonElement>(null);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);
  useEffect(() => {
    dialog.current?.showModal();
    cancel.current?.focus();
  }, []);
  async function remove() {
    setBusy(true);
    setError(null);
    const result = await removeLibrary(library.id);
    if (result.ok || result.error.kind === "libraryNotFound") {
      onRemoved(library.id);
      dialog.current?.close();
    } else {
      setError(result.error.message);
      setBusy(false);
    }
  }
  return (
    <dialog
      ref={dialog}
      className="remove-source-dialog"
      aria-labelledby="remove-source-title"
      aria-describedby="remove-source-description"
      onClose={onClose}
      onCancel={(event) => {
        if (busy) event.preventDefault();
      }}
    >
      <h2 id="remove-source-title">Remove {library.name}?</h2>
      <p id="remove-source-description">
        This removes the source, its indexed tracks, metadata edits and play
        counts from Muzo. Your music files stay where they are. Playlist entries
        remain, marked unavailable.
      </p>
      <code>{library.location}</code>
      {error && <p role="alert">{error}</p>}
      <div className="remove-source-dialog__actions">
        <button
          type="button"
          ref={cancel}
          disabled={busy}
          onClick={() => dialog.current?.close()}
        >
          Cancel
        </button>
        <button
          type="button"
          className="remove-source-dialog__confirm"
          disabled={busy}
          onClick={remove}
        >
          {busy ? "Removing..." : "Remove source"}
        </button>
      </div>
    </dialog>
  );
}
