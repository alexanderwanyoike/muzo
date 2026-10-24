import { ListMusic, Search } from "lucide-react";
import type { PlaylistDto } from "./api";

interface Props {
  playlists: PlaylistDto[] | null;
  selectedId: string | null;
  onSelect: (id: string) => void;
}

export default function PlaylistNavigator({ playlists, selectedId, onSelect }: Props) {
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
          <input type="search" aria-label="Search playlists" placeholder="Search playlists" />
        </div>
        <ul className="library-list" aria-label="Playlists">
          {playlists?.map((playlist) => (
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
