import { type FormEvent, useEffect, useState } from "react";
import {
  addDropboxLibrary,
  dropboxSyncStatus,
  setDropboxSyncInterval,
} from "./dropbox-api";
import type { LibraryDto } from "./types";

export default function DropboxLibrarySettings({
  onAdded,
}: {
  onAdded: (library: LibraryDto) => void;
}) {
  const [name, setName] = useState("");
  const [location, setLocation] = useState("");
  const [minutes, setMinutes] = useState(5);
  const [busy, setBusy] = useState(false);
  const [savingInterval, setSavingInterval] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [syncMessage, setSyncMessage] = useState<string | null>(null);

  useEffect(() => {
    let cancelled = false;
    const load = async () => {
      const result = await dropboxSyncStatus();
      if (cancelled) return;
      if (result.ok) {
        setMinutes(result.value.intervalMinutes ?? 5);
        setSyncMessage(
          result.value.lastError ??
            (result.value.syncing ? "Syncing Dropbox..." : null),
        );
      } else setSyncMessage(result.error.message);
    };
    void load();
    const timer = setInterval(() => {
      void load();
    }, 5000);
    return () => {
      cancelled = true;
      clearInterval(timer);
    };
  }, []);

  async function add(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    setBusy(true);
    setError(null);
    const result = await addDropboxLibrary({
      name: name.trim(),
      location: location.trim(),
    });
    if (result.ok) {
      setName("");
      setLocation("");
      onAdded(result.value);
    } else setError(result.error.message);
    setBusy(false);
  }

  async function changeInterval(value: number) {
    setSavingInterval(true);
    setError(null);
    const result = await setDropboxSyncInterval(value);
    if (result.ok) setMinutes(result.value.intervalMinutes);
    else setError(result.error.message);
    setSavingInterval(false);
  }

  return (
    <div>
      <form
        className="add-library-form"
        onSubmit={(event) => {
          void add(event);
        }}
      >
        <label className="add-library-form__field">
          Dropbox library name
          <input
            value={name}
            onChange={(event) => setName(event.target.value)}
            autoComplete="off"
            disabled={busy}
          />
        </label>
        <label className="add-library-form__field">
          Dropbox folder
          <input
            value={location}
            onChange={(event) => setLocation(event.target.value)}
            placeholder="/Music"
            autoComplete="off"
            disabled={busy}
          />
        </label>
        <button
          type="submit"
          disabled={busy || !name.trim() || !location.trim()}
        >
          Add Dropbox library
        </button>
      </form>
      <label>
        Sync Dropbox every
        <select
          value={minutes}
          disabled={savingInterval}
          onChange={(event) => {
            void changeInterval(Number(event.target.value));
          }}
        >
          {[1, 5, 15, 30, 60].map((value) => (
            <option key={value} value={value}>
              {value} {value === 1 ? "minute" : "minutes"}
            </option>
          ))}
        </select>
      </label>
      <p>
        Dropbox folders sync in the background. Use Refresh under Indexed
        sources to sync now.
      </p>
      {syncMessage && <p role="status">{syncMessage}</p>}
      {error && (
        <p role="alert" className="app__error">
          {error}
        </p>
      )}
    </div>
  );
}
