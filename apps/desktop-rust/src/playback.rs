use std::{fs::File, io, path::Path, time::Duration};

use rodio::{Decoder, Player, Source, decoder::DecoderError, mixer::Mixer, source::SeekError};

#[derive(Debug, thiserror::Error)]
pub enum PlaybackError {
    #[error("Could not open the audio file: {0}")]
    Open(#[from] io::Error),
    #[error("Could not decode the audio file: {0}")]
    Decode(#[from] DecoderError),
    #[error("Could not seek in the current track: {0}")]
    Seek(#[from] SeekError),
    #[error("Could not open an audio output: {0}")]
    Device(#[from] rodio::DeviceSinkError),
}

pub struct Playback {
    mixer: Mixer,
    player: Option<Player>,
    duration: Duration,
    volume: f32,
}

impl Playback {
    pub fn new(mixer: Mixer) -> Self {
        Self {
            mixer,
            player: None,
            duration: Duration::ZERO,
            volume: 1.0,
        }
    }

    pub fn play(&mut self, path: &Path) -> Result<(), PlaybackError> {
        let decoded = Decoder::try_from(File::open(path)?)?;
        let player = Player::connect_new(&self.mixer);
        player.set_volume(self.volume);
        self.duration = decoded.total_duration().unwrap_or_default();
        player.append(decoded);
        self.player = Some(player);
        Ok(())
    }

    pub fn reset(&mut self) {
        self.player = None;
        self.duration = Duration::ZERO;
    }

    pub fn is_finished(&self) -> bool {
        self.player.as_ref().is_some_and(Player::empty)
    }

    pub fn volume(&self) -> f32 {
        self.volume
    }

    pub fn set_volume(&mut self, value: f32) {
        self.volume = value;
        if let Some(player) = &self.player {
            player.set_volume(value);
        }
    }

    pub fn duration(&self) -> Duration {
        self.duration
    }

    pub fn position(&self) -> Duration {
        self.player.as_ref().map_or(Duration::ZERO, Player::get_pos)
    }

    pub fn is_paused(&self) -> bool {
        self.player.as_ref().is_none_or(Player::is_paused)
    }

    pub fn pause(&self) {
        if let Some(player) = &self.player {
            player.pause();
        }
    }

    pub fn resume(&self) {
        if let Some(player) = &self.player {
            player.play();
        }
    }

    pub fn seek(&self, target: Duration) -> Result<(), PlaybackError> {
        if let Some(player) = &self.player {
            player.try_seek(target.min(self.duration))?;
        }
        Ok(())
    }
}
