import type { BrowserWindow } from "electron";

export async function measureSidebarInteractions(
  window: BrowserWindow,
): Promise<unknown> {
  const read = (expression: string) =>
    window.webContents.executeJavaScript(expression) as Promise<unknown>;
  const settle = async () =>
    read(
      "new Promise(resolve => requestAnimationFrame(() => { document.getAnimations().filter(animation => animation.effect?.getComputedTiming().iterations !== Infinity).forEach(animation => animation.finish()); requestAnimationFrame(() => requestAnimationFrame(resolve)); }))",
    );
  const tab = (backwards = false) => {
    const modifiers = backwards ? ["shift"] : [];
    window.webContents.sendInputEvent({
      type: "keyDown",
      keyCode: "Tab",
      modifiers,
    });
    window.webContents.sendInputEvent({
      type: "keyUp",
      keyCode: "Tab",
      modifiers,
    });
  };
  async function hover(selector: string) {
    const point = (await read(
      `(() => { const rect = document.querySelector(${JSON.stringify(selector)}).getBoundingClientRect(); return {x:Math.round(rect.x+rect.width/2),y:Math.round(rect.y+rect.height/2)}; })()`,
    )) as { x: number; y: number };
    window.webContents.sendInputEvent({ type: "mouseMove", ...point });
    await settle();
    return read(
      `(() => { const style = getComputedStyle(document.querySelector(${JSON.stringify(selector)})); return {color:style.color, background:style.backgroundColor}; })()`,
    );
  }
  async function focus(selector: string) {
    await settle();
    return read(
      `(() => { const element = document.activeElement; const style = getComputedStyle(element); return {matches:element.matches(${JSON.stringify(selector)}),visible:element.matches(':focus-visible'),offset:style.outlineOffset,color:style.outlineColor}; })()`,
    );
  }
  await read("document.querySelector('.app-sidebar__nav-item').click()");
  await settle();
  await read("document.querySelector('.source-browser input').focus()");
  tab();
  const row = await focus(".library-list__row");
  const removal = await hover(".library-list__remove");
  await read(
    "document.querySelector('.app-sidebar__panel-slot:not([hidden]) .sidebar-panel__footer button').focus()",
  );
  tab(true);
  tab();
  const footer = await focus(".sidebar-panel__footer button");
  await read("document.querySelector('.library-list__remove').click()");
  await settle();
  const confirmation = await hover(".remove-source-dialog__confirm");
  await read(
    "document.querySelector('.remove-source-dialog__actions button').click()",
  );
  await settle();
  await read("document.querySelectorAll('.app-sidebar__nav-item')[1].click()");
  await settle();
  return { row, footer, removal, confirmation };
}
