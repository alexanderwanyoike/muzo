import { useEffect, useRef, useState, type FormEvent } from "react";
import type { Result } from "../shared/result";

interface Props {
  onCreate: (name: string) => Promise<Result<void>>;
  onClose: () => void;
}

export default function CreatePlaylistForm({ onCreate, onClose }: Props) {
  const [name, setName] = useState("");
  const [error, setError] = useState<string | null>(null);
  const [creating, setCreating] = useState(false);
  const input = useRef<HTMLInputElement>(null);
  useEffect(() => input.current?.focus(), []);
  async function submit(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    const trimmed = name.trim();
    if (!trimmed) {
      setError("Enter a playlist name.");
      return;
    }
    setCreating(true);
    setError(null);
    const result = await onCreate(trimmed);
    if (result.ok) onClose();
    else {
      setError(result.error.message);
      setCreating(false);
    }
  }
  return (
    <form className="playlist-panel__form" onSubmit={submit}>
      <label>
        Playlist name
        <input
          ref={input}
          value={name}
          onChange={(event) => setName(event.target.value)}
          onKeyDown={(event) => {
            if (event.key === "Escape" && !creating) {
              event.preventDefault();
              onClose();
            }
          }}
        />
      </label>
      <div className="playlist-panel__form-actions">
        <button type="submit" disabled={creating}>
          {creating ? "Creating..." : "Create playlist"}
        </button>
        <button type="button" disabled={creating} onClick={onClose}>
          Cancel
        </button>
      </div>
      {error && (
        <p role="alert" className="playlist-panel__error">
          {error}
        </p>
      )}
    </form>
  );
}
