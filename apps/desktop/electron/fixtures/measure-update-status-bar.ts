export function measureUpdateStatusBar() {
  const button = document.querySelector<HTMLElement>(".update-notice button");
  const footer = document.querySelector<HTMLElement>(
    'footer[aria-label="Application status"]',
  );
  const sidebar = document.querySelector<HTMLElement>(".app-sidebar");
  const content = document.querySelector<HTMLElement>(".app-content");
  if (!button || !footer || !sidebar || !content)
    throw new Error("Application status bar missing");
  const rect = button.getBoundingClientRect();
  const bar = footer.getBoundingClientRect();
  const style = getComputedStyle(button);
  return {
    right: rect.right,
    left: rect.left,
    top: rect.top,
    bottom: rect.bottom,
    buttonHeight: rect.height,
    barLeft: bar.left,
    barRight: bar.right,
    barTop: bar.top,
    barBottom: bar.bottom,
    barHeight: bar.height,
    sidebarBottom: sidebar.getBoundingClientRect().bottom,
    contentBottom: content.getBoundingClientRect().bottom,
    width: window.innerWidth,
    height: window.innerHeight,
    overflow: document.documentElement.scrollWidth > window.innerWidth,
    focused: button.matches(":focus-visible"),
    outlineWidth: style.outlineWidth,
    outlineOffset: style.outlineOffset,
  };
}
