import SidebarPanel from "./sidebar-panel";
import CreatePlaylistForm from "./create-playlist-form";
import type { Result } from "../shared/result";
import { useEffect, useRef, useState } from "react";
import { ListMusic, Search, X, Plus } from "lucide-react";
import type { PlaylistDto } from "./api";

interface Props {
  playlists: PlaylistDto[] | null;
  selectedId: string | null;
  onSelect: (id: string) => void;
  onCreate: (name: string) => Promise<Result<void>>;
}

export default function PlaylistNavigator({ playlists, selectedId, onSelect, onCreate }: Props) {
  const [creating, setCreating] = useState(false);
  const addButton = useRef<HTMLButtonElement>(null);
  const restoreFocus = useRef(false);
  function closeCreate() { restoreFocus.current = true; setCreating(false); }
  useEffect(() => { if (!creating && restoreFocus.current) { restoreFocus.current = false; addButton.current?.focus(); } }, [creating]);
  const [query, setQuery] = useState("");
  const searchInput = useRef<HTMLInputElement>(null);
  const search = query.trim().toLowerCase();
  const visible = playlists?.filter((playlist) => playlist.name.toLowerCase().includes(search));
  function clearSearch() { setQuery(""); searchInput.current?.focus(); }
  return (
    <SidebarPanel title="Playlists" headingId="playlists-panel-heading" icon={<ListMusic aria-hidden="true" />} count={playlists?.length ?? null} footer={
        creating ? <CreatePlaylistForm onCreate={async (name) => { const result = await onCreate(name); if (result.ok) setQuery(""); return result; }} onClose={closeCreate} /> : <button ref={addButton} type="button" disabled={playlists === null} onClick={() => setCreating(true)}><Plus aria-hidden="true" /> New playlist</button>
    }>
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
    </SidebarPanel>
  );
}
