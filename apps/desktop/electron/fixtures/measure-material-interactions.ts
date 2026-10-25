import type { BrowserWindow } from "electron";
import { measureSidebarInteractions } from "./measure-sidebar-interactions";

export async function measureMaterialInteractions(
  window: BrowserWindow,
): Promise<unknown> {
  const read = async (expression: string) =>
    window.webContents.executeJavaScript(expression) as Promise<unknown>;
  const settle = async () =>
    read(
      "document.fonts.ready.then(() => new Promise(resolve => requestAnimationFrame(() => { document.getAnimations().filter(animation => animation.effect?.getComputedTiming().iterations !== Infinity).forEach(animation => animation.finish()); requestAnimationFrame(() => requestAnimationFrame(resolve)); })))",
    );
  await settle();
  const initial = (await read(`(() => {
    const button = document.querySelector('.app-sidebar__nav-item');
    const rect = button.getBoundingClientRect();
    return {mode: document.documentElement.dataset.mode, foreground: getComputedStyle(button).color,
      selected: getComputedStyle(document.querySelector('.app-sidebar__nav-item--active')).backgroundColor,
      x:Math.round(rect.x+rect.width/2), y:Math.round(rect.y+rect.height/2)};
  })()`)) as {
    mode: string;
    foreground: string;
    selected: string;
    x: number;
    y: number;
  };
  window.webContents.focus();
  window.webContents.sendInputEvent({
    type: "mouseMove",
    x: initial.x,
    y: initial.y,
  });
  await settle();
  const hover = await read(
    `(() => { const style = getComputedStyle(document.querySelector('.app-sidebar__nav-item')); return {background: style.backgroundColor, color: style.color}; })()`,
  );
  await read("document.querySelector('.app-sidebar__nav-item').focus()");
  window.webContents.sendInputEvent({ type: "keyDown", keyCode: "Tab" });
  window.webContents.sendInputEvent({ type: "keyUp", keyCode: "Tab" });
  await settle();
  const focus = await read(
    `(() => { const button = document.activeElement; const style = getComputedStyle(button); return {visible:button.matches(':focus-visible'), color:style.outlineColor, style:style.outlineStyle}; })()`,
  );
  const sidebar = await measureSidebarInteractions(window);
  window.webContents.sendInputEvent({ type: "mouseMove", x: 300, y: 100 });
  await read("document.activeElement.blur()");
  await settle();
  return { ...initial, hover, focus, sidebar };
}
