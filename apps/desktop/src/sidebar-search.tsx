import { Search, X } from "lucide-react";
import { useRef } from "react";

interface Props {
  label: string;
  clearLabel: string;
  value: string;
  onChange: (value: string) => void;
}

export default function SidebarSearch({ label, clearLabel, value, onChange }: Props) {
  const input = useRef<HTMLInputElement>(null);
  function clear() { onChange(""); input.current?.focus(); }
  return (
    <div className="source-browser__search">
      <Search aria-hidden="true" />
      <input ref={input} type="search" aria-label={label} placeholder={label} value={value} onChange={(event) => onChange(event.target.value)} onKeyDown={(event) => { if (event.key === "Escape") clear(); }} />
      {value && <button type="button" aria-label={clearLabel} onClick={clear}><X aria-hidden="true" /></button>}
    </div>
  );
}
