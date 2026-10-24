import type { ReactNode } from "react";

interface Props {
  title: string;
  headingId: string;
  icon: ReactNode;
  count: number | null;
  footer: ReactNode;
  children: ReactNode;
}

export default function SidebarPanel({ title, headingId, icon, count, footer, children }: Props) {
  return (
    <section className="sidebar-panel" aria-labelledby={headingId}>
      <header className="sidebar-panel__header">
        {icon}
        <h2 id={headingId}>{title}</h2>
        <span className="sidebar-panel__total" aria-live="polite">{count ?? "..."}</span>
      </header>
      {children}
      <footer className="sidebar-panel__footer">{footer}</footer>
    </section>
  );
}
