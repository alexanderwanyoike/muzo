import { useRef, useState } from "react";
import { ListMusic, Search, X } from "lucide-react";
import type { PlaylistDto } from "./api";

interface Props {
  playlists: PlaylistDto[] | null;
  selectedId: string | null;
  onSelect: (id: string) => void;
}

export default function PlaylistNavigator({ playlists, selectedId, onSelect }: Props) {
  const [query, setQuery] = useState("");
  const searchInput = useRef<HTMLInputElement>(null);
  const search = query.trim().toLowerCase();
  const visible = playlists?.filter((playlist) => playlist.name.toLowerCase().includes(search));
  function clearSearch() { setQuery(""); searchInput.current?.focus(); }
  return (
    <section className="sources-panel" aria-labelledby="playlists-panel-heading">
      <header className="sources-panel__header">
        <ListMusic aria-hidden="true" />
        <h2 id="playlists-panel-heading">Playlists</h2>
        <span className="sources-panel__total" aria-live="polite">{playlists?.length ?? "..."}</span>
      </header>
      <div className="source-browser">
        <div className="source-browser__search">
          <Search aria-hidden="true" />
          <input ref={searchInput} type="search" aria-label="Search playlists" placeholder="Search playlists" value={query} onChange={(event) => setQuery(event.target.value)} onKeyDown={(event) => { if (event.key === "Escape") clearSearch(); }} />
          {query && <button type="button" aria-label="Clear playlist search" onClick={clearSearch}><X aria-hidden="true" /></button>}
        </div>
        {query && <p className="source-browser__count" aria-live="polite">{visible?.length ?? 0} of {playlists?.length ?? 0} playlists</p>}
        {visible?.length === 0 && search && <p className="library-list__empty">No playlists match your search.</p>}
        <ul className="library-list" aria-label="Playlists">
          {visible?.map((playlist) => (
            <li key={playlist.id} className={selectedId === playlist.id ? "library-list__item library-list__item--selected" : "library-list__item"}>
              <button type="button" className="library-list__row" aria-label={`Open playlist ${playlist.name}`} aria-pressed={selectedId === playlist.id} onClick={() => onSelect(playlist.id)} title={playlist.name}>
                <ListMusic aria-hidden="true" />
                <span className="library-list__name">{playlist.name}</span>
                <span className="library-list__count" aria-label={`${playlist.entries.length} tracks`}>{playlist.entries.length}</span>
              </button>
            </li>
          ))}
        </ul>
      </div>
    </section>
  );
}
