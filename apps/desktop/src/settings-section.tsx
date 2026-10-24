import type { ReactNode } from "react";

interface Props {
  title: string;
  headingId: string;
  description: string;
  className?: string;
  accessory?: ReactNode;
  children: ReactNode;
}

export default function SettingsSection({
  title,
  headingId,
  description,
  className = "",
  accessory,
  children,
}: Props) {
  return (
    <section
      className={`settings-view__section ${className}`}
      aria-labelledby={headingId}
    >
      <header className="settings-section__header">
        <div>
          <h3 id={headingId}>{title}</h3>
          <p>{description}</p>
        </div>
        {accessory}
      </header>
      <div className="settings-section__body">{children}</div>
    </section>
  );
}
