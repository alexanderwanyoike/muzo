interface Props {
  width: number;
  minimum: number;
  maximum: number;
  onCommit: (width: number) => void;
}

const keyboardStep = 8;
export default function SidebarSplitter({ width, minimum, maximum, onCommit }: Props) {
  return <div className="sidebar-splitter" role="separator" tabIndex={0} aria-orientation="vertical" aria-labelledby="sources-panel-heading" aria-controls="source-sidebar" aria-valuemin={minimum} aria-valuemax={maximum} aria-valuenow={width} aria-valuetext={`${width} pixels`} title="Resize Sources" onKeyDown={(event) => {
    const next = event.key === "ArrowLeft" ? width - keyboardStep : event.key === "ArrowRight" ? width + keyboardStep : event.key === "Home" ? minimum : event.key === "End" ? maximum : null;
    if (next !== null) { event.preventDefault(); onCommit(next); }
  }} />;
}
