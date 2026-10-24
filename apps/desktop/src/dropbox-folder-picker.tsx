import { useEffect, useState } from "react";
import type { DropboxFolder } from "../shared/dropbox-contracts";
import type { LibraryDto } from "./types";
import { addDropboxLibrary, listDropboxFolders } from "./dropbox-api";

export default function DropboxFolderPicker({
  accountId,
  onAdded,
  onClose,
}: {
  accountId: string;
  onAdded: (library: LibraryDto) => void;
  onClose: () => void;
}) {
  const [folders, setFolders] = useState<DropboxFolder[] | null>(null);
  const [selected, setSelected] = useState<DropboxFolder | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);

  useEffect(() => {
    let cancelled = false;
    void listDropboxFolders(accountId, "/").then((result) => {
      if (cancelled) return;
      if (result.ok) setFolders(result.value.folders);
      else setError(result.error.message);
    });
    return () => {
      cancelled = true;
    };
  }, [accountId]);

  async function addSelected() {
    if (!selected || busy) return;
    setBusy(true);
    const result = await addDropboxLibrary({
      name: selected.name,
      location: selected.path,
      accountId,
    });
    if (result.ok) {
      onAdded(result.value);
      setSelected(null);
    } else setError(result.error.message);
    setBusy(false);
  }

  return (
    <div
      className="dropbox-picker"
      role="region"
      aria-label="Dropbox folder browser"
    >
      <div className="dropbox-picker__toolbar">
        <strong>Dropbox folders</strong>
        <button type="button" disabled={busy} onClick={onClose}>
          Close folder browser
        </button>
      </div>
      <p>Choose folders to add. Their subfolders are included.</p>
      {folders === null && !error && <p role="status">Loading folders...</p>}
      <ul>
        {folders?.map((folder) => (
          <li key={folder.id}>
            <label>
              <input
                type="checkbox"
                aria-label={`Select ${folder.name}`}
                checked={selected?.id === folder.id}
                disabled={busy}
                onChange={() =>
                  setSelected(selected?.id === folder.id ? null : folder)
                }
              />
              {folder.name}
            </label>
          </li>
        ))}
      </ul>
      {error && <p role="alert">{error}</p>}
      <button
        type="button"
        disabled={!selected || busy}
        onClick={() => {
          void addSelected();
        }}
      >
        {busy ? "Adding folder..." : "Add 1 folder"}
      </button>
    </div>
  );
}
