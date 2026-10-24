import type { PlaylistDto, PlaylistEntryDto, TrackDto } from "./api";
import type { PlaybackQueueItem } from "./useAudioPlayer";

export function entryLabel(entry: PlaylistEntryDto): string {
  return entry.trackTitle ?? entry.trackId;
}

export function playButtonLabel(
  entry: PlaylistEntryDto,
  playlist: PlaylistDto,
  currentTrackId: string | null,
  isPlaying: boolean,
  currentQueueEntryId: string | null | undefined,
): string {
  const label = entryLabel(entry);
  if (!trackFromEntry(entry)) {
    return `Track ${label} is unavailable in ${playlist.name}`;
  }
  if (isCurrentEntry(entry, currentTrackId, currentQueueEntryId) && isPlaying) {
    return `Pause ${label} from ${playlist.name}`;
  }
  return `Play ${label} from ${playlist.name}`;
}

export function trackFromEntry(entry: PlaylistEntryDto): TrackDto | null {
  if (!entry.trackLibraryId) {
    return null;
  }

  return {
    id: entry.trackId,
    libraryId: entry.trackLibraryId,
    title: entryLabel(entry),
    artist: entry.trackArtist ?? "Unknown Artist",
    album: null,
    trackNumber: null,
    discNumber: null,
    genre: null,
    year: null,
    metadataOverridden: false,
    durationSeconds: entry.trackDurationSeconds ?? 0,
    filePath: "",
  };
}

export function playableEntries(playlist: PlaylistDto): PlaybackQueueItem[] {
  return playlist.entries.flatMap((entry) => {
    const track = trackFromEntry(entry);
    return track ? [{ entryId: entry.id, track }] : [];
  });
}

export function isCurrentEntry(entry: PlaylistEntryDto, trackId: string | null, queueEntryId: string | null | undefined): boolean {
  return queueEntryId === undefined ? entry.trackId === trackId : entry.id === queueEntryId;
}
