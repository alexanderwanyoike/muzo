use std::time::Duration;

use std::{error::Error, path::Path};

use crate::track::Track;

pub type AudioError = Box<dyn Error + Send + Sync>;

pub trait AudioPlayback {
    fn play(&mut self, path: &Path) -> Result<(), AudioError>;
    fn reset(&mut self);
    fn is_finished(&self) -> bool;
    fn volume(&self) -> f32;
    fn set_volume(&mut self, value: f32);
    fn duration(&self) -> Duration;
    fn position(&self) -> Duration;
    fn is_paused(&self) -> bool;
    fn pause(&self);
    fn resume(&self);
    fn seek(&mut self, target: Duration) -> Result<(), AudioError>;
}

#[derive(Debug, thiserror::Error)]
pub enum SessionError {
    #[error("This queue entry is no longer available.")]
    MissingEntry,
    #[error(transparent)]
    Playback(#[from] AudioError),
}

pub struct Session {
    playback: Box<dyn AudioPlayback>,
    tracks: Vec<Track>,
    current: Option<usize>,
    started: bool,
}

impl Session {
    pub fn new(playback: impl AudioPlayback + 'static) -> Self {
        Self {
            playback: Box::new(playback),
            tracks: Vec::new(),
            current: None,
            started: false,
        }
    }

    pub fn extend(&mut self, tracks: impl IntoIterator<Item = Track>) {
        self.tracks.extend(tracks);
    }

    pub fn replace(&mut self, tracks: Vec<Track>) {
        self.playback.reset();
        self.tracks = tracks;
        self.current = None;
        self.started = false;
    }

    pub fn tracks(&self) -> &[Track] {
        &self.tracks
    }

    pub fn current_index(&self) -> Option<usize> {
        self.current
    }

    pub fn current(&self) -> Option<&Track> {
        self.current.and_then(|index| self.tracks.get(index))
    }

    pub fn select(&mut self, index: usize) -> Result<(), SessionError> {
        if self.current == Some(index) && self.started {
            return Ok(());
        }
        let track = self.tracks.get(index).ok_or(SessionError::MissingEntry)?;
        self.current = Some(index);
        self.started = false;
        self.playback.reset();
        self.playback.play(track.path())?;
        self.started = true;
        Ok(())
    }

    pub fn next_track(&mut self) -> Result<(), SessionError> {
        if let Some(index) = self.current.map(|index| index + 1)
            && index < self.tracks.len()
        {
            self.select(index)?;
        }
        Ok(())
    }

    pub fn previous_track(&mut self) -> Result<(), SessionError> {
        if let Some(index) = self.current.and_then(|index| index.checked_sub(1)) {
            self.select(index)?;
        }
        Ok(())
    }

    pub fn advance_if_finished(&mut self) -> Result<(), SessionError> {
        if self.started && self.playback.is_finished() {
            self.started = false;
            self.next_track()?;
        }
        Ok(())
    }

    pub fn toggle(&mut self) -> Result<(), SessionError> {
        if let Some(index) = self.current {
            if !self.started {
                self.select(index)?;
            } else if self.playback.is_paused() {
                self.playback.resume();
            } else {
                self.playback.pause();
            }
        }
        Ok(())
    }

    pub fn is_playing(&self) -> bool {
        self.started && !self.playback.is_paused()
    }

    pub fn observe(&self) -> (Duration, Duration) {
        (self.playback.position(), self.playback.duration())
    }

    pub fn seek(&mut self, value: Duration) -> Result<(), SessionError> {
        self.playback.seek(value)?;
        Ok(())
    }

    pub fn volume(&self) -> f32 {
        self.playback.volume()
    }

    pub fn set_volume(&mut self, value: f32) {
        self.playback.set_volume(value);
    }
}
