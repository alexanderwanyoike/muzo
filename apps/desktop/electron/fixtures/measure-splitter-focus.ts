export function measureSplitterFocus(accents: string[]) {
  const splitter = document.querySelector<HTMLElement>(".sidebar-splitter");
  if (
    !splitter ||
    splitter !== document.activeElement ||
    !splitter.matches(":focus-visible")
  )
    throw new Error("The sidebar resize handle did not receive keyboard focus");
  const canvas = document.createElement("canvas");
  canvas.width = 1;
  canvas.height = 1;
  const context = canvas.getContext("2d");
  if (!context) throw new Error("Focus colour decoder unavailable");
  const root = document.documentElement;
  const original = root.dataset.accent;
  try {
    return accents.map((id) => {
      root.dataset.accent = id;
      context.fillStyle = getComputedStyle(splitter, "::after").backgroundColor;
      context.fillRect(0, 0, 1, 1);
      const [red, green, blue] = context.getImageData(0, 0, 1, 1).data;
      return {
        id,
        colour: `#${[red, green, blue].map((channel) => channel.toString(16).padStart(2, "0")).join("")}`,
      };
    });
  } finally {
    root.dataset.accent = original ?? "cobalt";
  }
}
