import type { Rectangle } from "electron";

export const MINI_PLAYER_CORNER_RADIUS = 14;

export function roundedWindowShape(width: number, height: number): Rectangle[] {
  const radius = Math.min(
    MINI_PLAYER_CORNER_RADIUS,
    Math.floor(width / 2),
    Math.floor(height / 2),
  );
  const rows: Rectangle[] = [];
  for (let y = 0; y < radius; y += 1) {
    const distance = radius - y - 0.5;
    const inset = Math.ceil(
      radius - Math.sqrt(radius * radius - distance * distance),
    );
    rows.push({ x: inset, y, width: width - inset * 2, height: 1 });
    rows.push({
      x: inset,
      y: height - y - 1,
      width: width - inset * 2,
      height: 1,
    });
  }
  rows.push({ x: 0, y: radius, width, height: height - radius * 2 });
  return rows;
}
