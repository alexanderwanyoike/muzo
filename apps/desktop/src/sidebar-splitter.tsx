import { useRef, useState, type PointerEvent } from "react";

interface Props {
  width: number;
  minimum: number;
  maximum: number;
  onPreview: (width: number) => void;
  onCommit: (width: number) => void;
}
interface Drag {
  pointer: number;
  x: number;
  startWidth: number;
  latestWidth: number;
}

const keyboardStep = 8;
export default function SidebarSplitter({ width, minimum, maximum, onPreview, onCommit }: Props) {
  const drag = useRef<Drag | null>(null);
  const [dragging, setDragging] = useState(false);
  function cancel(element: HTMLDivElement) {
    const active = drag.current;
    if (!active) return;
    drag.current = null;
    setDragging(false);
    onPreview(active.startWidth);
    if (element.hasPointerCapture?.(active.pointer)) element.releasePointerCapture(active.pointer);
  }
  function finish(event: PointerEvent<HTMLDivElement>) {
    const active = drag.current;
    if (!active || active.pointer !== event.pointerId) return;
    drag.current = null;
    setDragging(false);
    onCommit(active.latestWidth);
    if (event.currentTarget.hasPointerCapture?.(event.pointerId)) event.currentTarget.releasePointerCapture(event.pointerId);
  }
  return <div className="sidebar-splitter" role="separator" tabIndex={0} aria-orientation="vertical" aria-labelledby="sources-panel-heading" aria-controls="source-sidebar" aria-valuemin={minimum} aria-valuemax={maximum} aria-valuenow={width} aria-valuetext={`${width} pixels`} title="Resize Sources" data-resizing={dragging || undefined}
    onPointerDown={(event) => {
      if (event.button !== 0 || drag.current) return;
      event.preventDefault();
      event.currentTarget.focus();
      drag.current = { pointer: event.pointerId, x: event.clientX, startWidth: width, latestWidth: width };
      setDragging(true);
      event.currentTarget.setPointerCapture?.(event.pointerId);
    }}
    onPointerMove={(event) => {
      const active = drag.current;
      if (!active || active.pointer !== event.pointerId) return;
      active.latestWidth = active.startWidth + event.clientX - active.x;
      onPreview(active.latestWidth);
    }}
    onPointerUp={finish}
    onPointerCancel={(event) => { if (drag.current?.pointer === event.pointerId) cancel(event.currentTarget); }}
    onLostPointerCapture={(event) => { if (drag.current?.pointer === event.pointerId) cancel(event.currentTarget); }}
    onKeyDown={(event) => {
      if (event.key === "Escape" && drag.current) { event.preventDefault(); cancel(event.currentTarget); return; }
      const next = event.key === "ArrowLeft" ? width - keyboardStep : event.key === "ArrowRight" ? width + keyboardStep : event.key === "Home" ? minimum : event.key === "End" ? maximum : null;
      if (next !== null) { event.preventDefault(); onCommit(next); }
    }} />;
}
