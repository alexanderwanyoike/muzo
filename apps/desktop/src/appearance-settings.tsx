import { ACCENTS, MATERIALS, MODES, type Appearance } from "./appearance";
import "./appearance-settings.css";

interface Props {
  appearance: Appearance;
  onChange: (appearance: Appearance) => void;
  error: string | null;
}
export default function AppearanceSettings({
  appearance,
  onChange,
  error,
}: Props) {
  return (
    <section
      className="settings-view__section appearance-settings"
      aria-labelledby="appearance-heading"
    >
      <header>
        <h3 id="appearance-heading">Appearance</h3>
        <p>Choose a material, appearance and accent colour.</p>
      </header>
      <fieldset>
        <legend>Material</legend>
        <div className="appearance-materials">
          {MATERIALS.map((material) => (
            <label className="appearance-material" key={material.id}>
              <input
                type="radio"
                name="material"
                value={material.id}
                checked={appearance.theme === material.id}
                onChange={() => onChange({ ...appearance, theme: material.id })}
                aria-label={material.name}
              />
              <span
                className="appearance-preview"
                data-theme={material.id}
                data-mode={appearance.mode}
                aria-hidden="true"
              >
                <span className="appearance-preview__chrome" />
                <span className="appearance-preview__sidebar">
                  <i />
                  <i />
                  <i />
                </span>
                <span className="appearance-preview__songs">
                  <i />
                  <i />
                  <i />
                </span>
              </span>
              <span className="appearance-material__name">{material.name}</span>
              <small>{material.detail}</small>
            </label>
          ))}
        </div>
      </fieldset>
      <div className="appearance-settings__row">
        <fieldset>
          <legend>Appearance</legend>
          <div className="appearance-modes">
            {MODES.map((mode) => (
              <label key={mode}>
                <input
                  type="radio"
                  name="mode"
                  checked={appearance.mode === mode}
                  onChange={() => onChange({ ...appearance, mode })}
                />
                <span>{mode === "light" ? "Light" : "Dark"}</span>
              </label>
            ))}
          </div>
        </fieldset>
        <fieldset>
          <legend>Accent colour</legend>
          <div className="appearance-accents">
            {ACCENTS.map((accent) => (
              <label
                key={accent.id}
                data-accent={accent.id}
                title={accent.name}
              >
                <input
                  type="radio"
                  name="accent"
                  checked={appearance.accent === accent.id}
                  onChange={() =>
                    onChange({ ...appearance, accent: accent.id })
                  }
                  aria-label={accent.name}
                />
                <span aria-hidden="true" />
              </label>
            ))}
          </div>
        </fieldset>
      </div>
      {error && <p role="alert">{error}</p>}
    </section>
  );
}
