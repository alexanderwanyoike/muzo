import { useEffect, useRef, useState } from "react";
import { Disc3 } from "lucide-react";
import type { TrackDto } from "./api";
import { readTrackArtwork } from "./artwork-api";
import "./track-artwork.css";

interface Props {
  track: Pick<TrackDto, "id" | "libraryId" | "title" | "album">;
  eager?: boolean;
  className?: string;
}
export default function TrackArtwork({
  track,
  eager = false,
  className = "",
}: Props) {
  const element = useRef<HTMLSpanElement>(null);
  const [visible, setVisible] = useState(eager);
  const [loaded, setLoaded] = useState<{
    key: string;
    uri: string | null;
    error: string | null;
  } | null>(null);
  const key = JSON.stringify([track.libraryId, track.id]);
  const label = track.album ?? track.title;
  useEffect(() => {
    if (eager || !("IntersectionObserver" in window)) {
      setVisible(true);
      return undefined;
    }
    const observer = new IntersectionObserver(
      (entries) => {
        if (entries.some((entry) => entry.isIntersecting)) {
          setVisible(true);
          observer.disconnect();
        }
      },
      { rootMargin: "150px" },
    );
    if (element.current) observer.observe(element.current);
    return () => observer.disconnect();
  }, [eager]);
  useEffect(() => {
    if (!visible) return undefined;
    let cancelled = false;
    void readTrackArtwork(track.libraryId, track.id).then((result) => {
      if (cancelled) return;
      setLoaded({
        key,
        uri: result.ok ? result.value : null,
        error: result.ok ? null : result.error.message,
      });
    });
    return () => {
      cancelled = true;
    };
  }, [key, track.id, track.libraryId, visible]);
  const artwork = loaded?.key === key ? loaded : null;
  return (
    <span
      ref={element}
      className={`track-artwork ${className}`}
      title={
        artwork?.error ?? (artwork?.uri ? label : "No embedded album artwork")
      }
    >
      {artwork?.uri ? (
        <img
          src={artwork.uri}
          alt={`Artwork for ${label}`}
          decoding="async"
          onError={() =>
            setLoaded({
              key,
              uri: null,
              error: "Album artwork could not be displayed.",
            })
          }
        />
      ) : (
        <Disc3 aria-hidden="true" />
      )}
    </span>
  );
}
