import { useState, type FormEvent } from "react";
import { addLibrary } from "./api";
import { openDirectory } from "./runtime-bridge";
import { errorMessage, type AddLibraryErrorDto, type LibraryDto } from "./types";

interface AddLibraryFormProps {
  onAdded: (library: LibraryDto) => void;
}

export default function AddLibraryForm({ onAdded }: AddLibraryFormProps) {
  const [name, setName] = useState("");
  const [location, setLocation] = useState("");
  const [error, setError] = useState<string | null>(null);
  const [submitting, setSubmitting] = useState(false);

  async function handleBrowse() {
    setError(null);
    try {
      const selected = await openDirectory();
      if (selected) {
        setLocation(selected);
        if (name.trim() === "") {
          setName(selected.split(/[\\/]/).filter(Boolean).at(-1) ?? selected);
        }
      }
    } catch {
      setError(
        "Could not open the folder picker. If this is running in a browser, start Muzo with yarn desktop:dev.",
      );
    }
  }

  async function handleSubmit(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    setError(null);
    setSubmitting(true);
    try {
      const created = await addLibrary({
        name,
        kind: "filesystem",
        location,
      });
      onAdded(created);
      setName("");
      setLocation("");
    } catch (err) {
      setError(errorMessage(err as AddLibraryErrorDto));
    } finally {
      setSubmitting(false);
    }
  }

  return (
    <form className="add-library-form" onSubmit={handleSubmit}>
      <label className="add-library-form__field">
        <span>Name</span>
        <input
          value={name}
          onChange={(e) => setName(e.target.value)}
          aria-label="Name"
          autoComplete="off"
        />
      </label>

      <label className="add-library-form__field">
        <span>Location</span>
        <input
          value={location}
          onChange={(e) => setLocation(e.target.value)}
          aria-label="Location"
          autoComplete="off"
        />
        <button type="button" onClick={handleBrowse}>
          Browse
        </button>
      </label>

      <button type="submit" disabled={submitting}>
        {submitting ? "Adding..." : "Add library"}
      </button>

      {error && (
        <p role="alert" className="add-library-form__error">
          {error}
        </p>
      )}
    </form>
  );
}
