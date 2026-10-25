export async function measureGlowControls(themes: string[], accents: string[]) {
  const root = document.documentElement;
  const deadline = Date.now() + 5000;
  while (!document.querySelector(".now-playing-bar__play")) {
    if (Date.now() > deadline)
      throw new Error("Accent controls did not render");
    await new Promise((resolve) => {
      setTimeout(resolve, 20);
    });
  }
  const stage = document.getElementById("stage");
  const play = document.querySelector<HTMLElement>(".now-playing-bar__play");
  const volume = document.querySelector<HTMLElement>(
    ".now-playing-bar__volume-slider",
  );
  if (!stage || !play || !volume) throw new Error("Accent controls missing");
  const canvas = document.createElement("canvas");
  canvas.width = 1;
  canvas.height = 1;
  const context = canvas.getContext("2d");
  if (!context) throw new Error("Accent colour decoder unavailable");
  function colour(value: string): string {
    if (!context) throw new Error("Accent colour decoder unavailable");
    context.clearRect(0, 0, 1, 1);
    context.fillStyle = value;
    context.fillRect(0, 0, 1, 1);
    const [red, green, blue, alpha] = context.getImageData(0, 0, 1, 1).data;
    if (alpha !== 255)
      throw new Error(`Expected opaque control colour: ${value}`);
    return `#${[red, green, blue].map((channel) => channel.toString(16).padStart(2, "0")).join("")}`;
  }
  async function settle() {
    await new Promise<void>((resolve) => {
      requestAnimationFrame(() => {
        document
          .getAnimations()
          .filter(
            (animation) =>
              animation.effect?.getComputedTiming().iterations !== Infinity,
          )
          .forEach((animation) => animation.finish());
        requestAnimationFrame(() => requestAnimationFrame(() => resolve()));
      });
    });
  }
  const measurements = [];
  const restored = [];
  for (const theme of themes)
    for (const mode of ["light", "dark"])
      for (const compact of [false, true]) {
        root.dataset.theme = theme;
        root.dataset.mode = mode;
        root.style.colorScheme = mode;
        stage.className = compact ? "compact-player" : "app-shell";
        root.dataset.accent = "cobalt";
        await settle();
        const original = {
          shadow: getComputedStyle(play).boxShadow,
          filter: getComputedStyle(volume).filter,
        };
        for (const accent of accents) {
          root.dataset.accent = accent;
          await settle();
          const style = getComputedStyle(play);
          measurements.push({
            theme,
            mode,
            compact,
            accent,
            foreground: colour(style.color),
            background: colour(style.backgroundColor),
            shadow: style.boxShadow,
            filter: getComputedStyle(volume).filter,
          });
        }
        root.dataset.accent = "cobalt";
        await settle();
        restored.push({
          original,
          after: {
            shadow: getComputedStyle(play).boxShadow,
            filter: getComputedStyle(volume).filter,
          },
        });
      }
  root.dataset.accent = "glow-green";
  await settle();
  return { measurements, restored };
}
