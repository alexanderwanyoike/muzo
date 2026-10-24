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
  const [path, setPath] = useState("/");
  const [folders, setFolders] = useState<DropboxFolder[] | null>(null);
  const [selected, setSelected] = useState<DropboxFolder[]>([]);
  const [loadError, setLoadError] = useState<string | null>(null);
  const [addError, setAddError] = useState<string | null>(null);
  const [message, setMessage] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);

  useEffect(() => {
    let cancelled = false;
    setFolders(null);
    setLoadError(null);
    void listDropboxFolders(accountId, path).then((result) => {
      if (cancelled) return;
      if (result.ok) setFolders(result.value.folders);
      else setLoadError(result.error.message);
    });
    return () => {
      cancelled = true;
    };
  }, [accountId, path]);

  function toggleFolder(folder: DropboxFolder) {
    setSelected((previous) =>
      previous.some((item) => item.path === folder.path)
        ? previous.filter((item) => item.path !== folder.path)
        : [...previous, folder],
    );
  }

  async function addSelected() {
    if (!selected.length || busy) return;
    setBusy(true);
    setAddError(null);
    setMessage(null);
    for (const folder of selected) {
      const result = await addDropboxLibrary({
        name: folder.name,
        location: folder.path,
        accountId,
      });
      if (!result.ok) {
        setAddError(result.error.message);
        setBusy(false);
        return;
      }
      onAdded(result.value);
      setSelected((previous) =>
        previous.filter((item) => item.path !== folder.path),
      );
    }
    setMessage(
      `Added ${selected.length} ${selected.length === 1 ? "folder" : "folders"}.`,
    );
    setBusy(false);
  }

  const loading = folders === null && !loadError;
  const segments = path.split("/").filter(Boolean);

  return (
    <section className="dropbox-picker" aria-label="Dropbox folder browser">
      <div className="dropbox-picker__toolbar">
        <nav
          className="dropbox-picker__breadcrumbs"
          aria-label="Dropbox folder location"
        >
          <button
            type="button"
            disabled={busy || loading || path === "/"}
            aria-label="Open Dropbox root"
            onClick={() => setPath("/")}
          >
            Dropbox
          </button>
          {segments.map((segment, index) => (
            <span key={segments.slice(0, index + 1).join("/")}>
              <span aria-hidden="true">/</span>
              <button
                type="button"
                disabled={busy || loading || index === segments.length - 1}
                onClick={() =>
                  setPath(`/${segments.slice(0, index + 1).join("/")}`)
                }
              >
                {segment}
              </button>
            </span>
          ))}
        </nav>
        <button
          className="dropbox-picker__close"
          type="button"
          disabled={busy}
          onClick={onClose}
          aria-label="Close folder browser"
        >
          Close
        </button>
      </div>
      <p className="dropbox-picker__hint">
        Select folders to add. Their subfolders are included.
      </p>
      {loading && (
        <p className="dropbox-picker__empty" role="status">
          Loading folders...
        </p>
      )}
      {loadError && <p role="alert">{loadError}</p>}
      {folders?.length === 0 && (
        <p className="dropbox-picker__empty">No subfolders here.</p>
      )}
      <ul className="dropbox-picker__folders">
        {folders?.map((folder) => (
          <li
            key={folder.id}
            className={
              selected.some((item) => item.path === folder.path)
                ? "dropbox-picker__folder dropbox-picker__folder--selected"
                : "dropbox-picker__folder"
            }
          >
            <input
              type="checkbox"
              aria-label={`Select ${folder.name}`}
              checked={selected.some((item) => item.path === folder.path)}
              disabled={busy}
              onChange={() => toggleFolder(folder)}
            />
            <span className="dropbox-picker__folder-icon" aria-hidden="true">
              <svg
                viewBox="0 0 24 24"
                fill="none"
                stroke="currentColor"
                strokeWidth="1.5"
              >
                <path d="M3 7a2 2 0 0 1 2-2h5l2 2h7a2 2 0 0 1 2 2v9a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2Z" />
              </svg>
            </span>
            <button
              className="dropbox-picker__open"
              type="button"
              aria-label={`Open ${folder.name}`}
              disabled={busy || loading}
              onClick={() => setPath(folder.path)}
            >
              <span>{folder.name}</span>
              <span aria-hidden="true">›</span>
            </button>
          </li>
        ))}
      </ul>
      <div className="dropbox-picker__selection">
        <span>
          {selected.length} {selected.length === 1 ? "folder" : "folders"}{" "}
          selected
        </span>
        {selected.length > 0 && (
          <ul className="dropbox-picker__chips">
            {selected.map((folder) => (
              <li key={folder.path}>
                <span title={folder.path}>{folder.name}</span>
                <button
                  type="button"
                  disabled={busy}
                  aria-label={`Remove ${folder.name} from selection`}
                  onClick={() => toggleFolder(folder)}
                >
                  ×
                </button>
              </li>
            ))}
          </ul>
        )}
        {addError && <p role="alert">{addError}</p>}
        {message && <p role="status">{message}</p>}
        <button
          className="dropbox-picker__add"
          type="button"
          disabled={!selected.length || busy}
          onClick={() => {
            void addSelected();
          }}
        >
          {busy
            ? "Adding folders..."
            : selected.length
              ? `Add ${selected.length} ${selected.length === 1 ? "folder" : "folders"}`
              : "Add selected folders"}
        </button>
      </div>
    </section>
  );
}
