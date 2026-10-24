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
  const [reload, setReload] = useState(0);
  const [folders, setFolders] = useState<DropboxFolder[] | null>(null);
  const [addedPaths, setAddedPaths] = useState<string[]>([]);
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
      if (result.ok) {
        setFolders(result.value.folders);
        setAddedPaths((previous) => [
          ...new Set([
            ...previous,
            ...result.value.addedPaths.map((item) => item.toLowerCase()),
          ]),
        ]);
      } else setLoadError(result.error.message);
    });
    return () => {
      cancelled = true;
    };
  }, [accountId, path, reload]);

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
    const failures: string[] = [];
    let added = 0;
    const outcomes = await Promise.all(
      selected.map(async (folder) => ({
        folder,
        result: await addDropboxLibrary({
          name: folder.name,
          location: folder.path,
          accountId,
        }),
      })),
    );
    for (const { folder, result } of outcomes) {
      if (!result.ok) {
        failures.push(`${folder.name}: ${result.error.message}`);
        continue;
      }
      onAdded(result.value);
      added += 1;
      setAddedPaths((previous) => [...previous, folder.path.toLowerCase()]);
      setSelected((previous) =>
        previous.filter((item) => item.path !== folder.path),
      );
    }
    if (added)
      setMessage(`Added ${added} ${added === 1 ? "folder" : "folders"}.`);
    if (failures.length) setAddError(failures.join(" "));
    setBusy(false);
  }

  const loading = folders === null && !loadError;
  const segments = path.split("/").filter(Boolean);
  const currentFolder = { id: path, name: segments.at(-1) ?? "Dropbox", path };

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
      {loadError && (
        <div className="dropbox-picker__error">
          <p role="alert">{loadError}</p>
          <button type="button" onClick={() => setReload((value) => value + 1)}>
            Retry folders
          </button>
        </div>
      )}
      {folders !== null && (
        <label className="dropbox-picker__current">
          <input
            type="checkbox"
            aria-label={
              path === "/" ? "Select entire Dropbox" : "Select current folder"
            }
            disabled={busy || addedPaths.includes(path.toLowerCase())}
            checked={selected.some((item) => item.path === path)}
            onChange={() => toggleFolder(currentFolder)}
          />
          <span>
            <strong>
              {path === "/" ? "Entire Dropbox" : currentFolder.name}
            </strong>
            <small>Include this folder and all its subfolders</small>
          </span>
          {addedPaths.includes(path.toLowerCase()) && (
            <span className="dropbox-picker__added">Added</span>
          )}
        </label>
      )}
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
              disabled={busy || addedPaths.includes(folder.path.toLowerCase())}
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
              {addedPaths.includes(folder.path.toLowerCase()) && (
                <span className="dropbox-picker__added">Added</span>
              )}
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
