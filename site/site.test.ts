import { existsSync, readFileSync, statSync } from "node:fs";
import { dirname, resolve, sep } from "node:path";
import { fileURLToPath } from "node:url";
import { beforeEach, describe, expect, it } from "vitest";

const root = dirname(fileURLToPath(import.meta.url));
const origin = "https://alexanderwanyoike.github.io";
const base = `${origin}/muzo/`;

function localFile(reference: string, parent = "index.html"): string {
  const projectReference =
    reference.startsWith("/") && !reference.startsWith("/muzo/")
      ? reference.slice(1)
      : reference;
  const url = new URL(projectReference, new URL(parent, base));
  expect(url.origin).toBe(origin);
  expect(url.pathname.startsWith("/muzo/")).toBe(true);
  const source = resolve(
    root,
    decodeURIComponent(url.pathname.slice("/muzo/".length)),
  );
  expect(source.startsWith(`${root}${sep}`)).toBe(true);
  const publicFile = resolve(
    root,
    "public",
    decodeURIComponent(url.pathname.slice("/muzo/".length)),
  );
  const file = existsSync(publicFile) ? publicFile : source;
  expect(statSync(file).isFile()).toBe(true);
  return file;
}

beforeEach(() => {
  document.documentElement.innerHTML = readFileSync(
    resolve(root, "index.html"),
    "utf8",
  );
});

describe("Muzo project site", () => {
  it("introduces the desktop preview with one main heading and working section navigation", () => {
    expect(document.querySelectorAll("h1")).toHaveLength(1);
    expect(document.querySelector("main")).not.toBeNull();
    expect(document.querySelector('a[href="#main"]')).not.toBeNull();
    for (const link of document.querySelectorAll('a[href^="#"]')) {
      expect(
        document.getElementById(link.getAttribute("href")?.slice(1) ?? ""),
      ).not.toBeNull();
    }
    expect(document.body.textContent?.replace(/\s+/g, " ")).toContain(
      "desktop preview",
    );
  });

  it("offers source installation and release history without inventing binary downloads", () => {
    expect(
      document.querySelector(
        'a[href="https://github.com/alexanderwanyoike/muzo/releases"]',
      ),
    ).not.toBeNull();
    expect(document.body.textContent?.replace(/\s+/g, " ")).toContain(
      "Packaged downloads are not published yet",
    );
    expect(document.body.textContent?.replace(/\s+/g, " ")).toContain(
      "yarn desktop:dev",
    );
    expect(document.querySelector("a[download]")).toBeNull();
    expect(document.querySelector('a[href*="/releases/download/"]')).toBeNull();
  });

  it("uses the current canonical screenshots with descriptive alternative text and dimensions", () => {
    for (const [name, width, height] of [
      ["muzo-player.jpg", 1440, 900],
      ["muzo-personalization.jpg", 1280, 900],
    ] as const) {
      const image = document.querySelector<HTMLImageElement>(
        `img[src="images/${name}"]`,
      );
      expect(image).not.toBeNull();
      expect(image?.alt.length).toBeGreaterThan(20);
      const capture = readFileSync(localFile(`images/${name}`));
      expect([...capture.subarray(0, 2)]).toEqual([0xff, 0xd8]);
      expect(Number(image?.getAttribute("width"))).toBe(width);
      expect(Number(image?.getAttribute("height"))).toBe(height);
    }
  });

  it("resolves every page asset and stylesheet resource beneath the GitHub Pages project base", () => {
    for (const element of document.querySelectorAll("[src], link[href]")) {
      const reference =
        element.getAttribute("src") ?? element.getAttribute("href") ?? "";
      if (element.getAttribute("rel") !== "canonical") {
        const file = localFile(reference);
        if (file.endsWith(".css")) {
          const css = readFileSync(file, "utf8");
          for (const match of css.matchAll(/url\(["']?([^"')]+)["']?\)/g)) {
            localFile(match[1] ?? "", reference);
          }
        }
      }
    }
  });

  it("provides canonical sharing metadata with a real project screenshot", () => {
    expect(
      document.querySelector('link[rel="canonical"]')?.getAttribute("href"),
    ).toBe(base);
    expect(document.querySelector('meta[name="viewport"]')).not.toBeNull();
    expect(
      document
        .querySelector('meta[name="description"]')
        ?.getAttribute("content")?.length,
    ).toBeGreaterThan(40);
    const share =
      document
        .querySelector('meta[property="og:image"]')
        ?.getAttribute("content") ?? "";
    expect(share.startsWith(base)).toBe(true);
    localFile(share);
    expect(document.querySelectorAll("script")).toHaveLength(0);
  });
});
