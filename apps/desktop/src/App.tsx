import { useEffect, useState } from "react";
import AddLibraryForm from "./AddLibraryForm";
import LibraryList from "./LibraryList";
import { listLibraries } from "./api";
import type { LibraryDto } from "./types";

interface ListError {
  kind: string;
  message?: string;
}

export default function App() {
  const [libraries, setLibraries] = useState<LibraryDto[] | null>(null);
  const [error, setError] = useState<string | null>(null);

  const refresh = async () => {
    setError(null);
    try {
      setLibraries(await listLibraries());
    } catch (err) {
      const e = err as ListError;
      setError(e.message ?? "Could not load libraries.");
    }
  };

  useEffect(() => {
    refresh();
  }, []);

  return (
    <main className="app">
      <header className="app__header">
        <h1>Muzo</h1>
        <p className="app__tagline">Your music, your libraries.</p>
      </header>

      <section className="app__section">
        <h2>Add a library</h2>
        <AddLibraryForm onAdded={refresh} />
      </section>

      <section className="app__section">
        <h2>Libraries</h2>
        {error && (
          <p role="alert" className="app__error">
            {error}
          </p>
        )}
        {libraries === null ? (
          <p className="app__placeholder">Loading...</p>
        ) : (
          <LibraryList libraries={libraries} />
        )}
      </section>
    </main>
  );
}
