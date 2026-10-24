export async function measureSettingsLayout() {
  const deadline = Date.now() + 5000;
  async function until(ready: () => boolean) {
    while (!ready()) {
      if (Date.now() > deadline) throw new Error("Settings fixture did not render");
      await new Promise((resolve) => { setTimeout(resolve, 20); });
    }
  }
  await until(() => !!document.querySelector(".app-sidebar__nav"));
  const settings = Array.from(document.querySelectorAll<HTMLButtonElement>(".app-sidebar__nav button")).find((button) => button.textContent?.trim() === "Settings");
  settings?.click();
  await until(() => document.querySelectorAll(".settings-view__section").length === 4 && (!!document.querySelector(".dropbox-sync") || !!document.querySelector(".dropbox-settings input")));
  await document.fonts.ready;
  const viewport = document.querySelector<HTMLElement>(".settings-view");
  if (!viewport) throw new Error("Settings scroll container missing");
  const sections = Array.from(document.querySelectorAll<HTMLElement>(".settings-view__section")).map((element) => {
    const rect = element.getBoundingClientRect();
    const heading = element.querySelector("h3")?.getBoundingClientRect();
    return { name: element.querySelector("h3")?.textContent, left: rect.left, width: rect.width, headingLeft: heading?.left };
  });
  const controls = Array.from(viewport.querySelectorAll<HTMLElement>(".add-library-form input, .add-library-form button, .dropbox-browse-button, .dropbox-sync select, .dropbox-settings__connection button, .settings-view__source-actions button")).map((element) => element.getBoundingClientRect().height);
  return { sections, controls, overflow: viewport.scrollWidth - viewport.clientWidth };
}
