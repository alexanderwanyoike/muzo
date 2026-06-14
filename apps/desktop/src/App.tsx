import { useState } from "react";
import AddLibraryForm from "./AddLibraryForm";
import type { LibraryDto } from "./types";

export default function App() {
  const [added, setAdded] = useState<LibraryDto | null>(null);

  return (
    <main className="app">
      <header className="app__header">
        <h1>Muzo</h1>
        <p className="app__tagline">Your music, your libraries.</p>
      </header>

      <section className="app__section">
        <h2>Add a library</h2>
        <AddLibraryForm onAdded={setAdded} />

        {added && (
          <p className="app__success" role="status">
            Added <strong>{added.name}</strong> at {added.location}.
          </p>
        )}
      </section>
    </main>
  );
}
