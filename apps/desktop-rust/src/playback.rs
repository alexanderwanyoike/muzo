use std::{
    fs::File,
    io,
    path::{Path, PathBuf},
    time::Duration,
};

use rodio::{Decoder, Player, Source, decoder::DecoderError, mixer::Mixer, source::SeekError};

#[derive(Debug, thiserror::Error)]
pub enum PlaybackError {
    #[error("Could not open the audio file: {0}")]
    Open(#[from] io::Error),
    #[error("Could not decode the audio file: {0}")]
    Decode(#[from] DecoderError),
    #[error("Could not seek in the current track: {0}")]
    Seek(#[from] SeekError),
    #[error("Could not decode the Ogg Opus file: {0}")]
    Opus(#[from] symphonia::core::errors::Error),
    #[error("Could not open an audio output: {0}")]
    Device(#[from] rodio::DeviceSinkError),
}

pub struct Playback {
    mixer: Mixer,
    player: Option<Player>,
    duration: Duration,
    offset: Duration,
    source_path: Option<PathBuf>,
    volume: f32,
}

impl Playback {
    pub fn new(mixer: Mixer) -> Self {
        Self {
            mixer,
            player: None,
            duration: Duration::ZERO,
            offset: Duration::ZERO,
            source_path: None,
            volume: 1.0,
        }
    }

    fn start(&mut self, path: &Path, offset: Duration, paused: bool) -> Result<(), PlaybackError> {
        let file = File::open(path)?;
        let decoded: Box<dyn Source + Send> = match Decoder::try_from(file) {
            Ok(mut decoder) => {
                if !offset.is_zero() {
                    decoder.try_seek(offset)?;
                }
                Box::new(decoder)
            }
            Err(error)
                if path.extension().is_some_and(|ext| {
                    ext.eq_ignore_ascii_case("opus") || ext.eq_ignore_ascii_case("ogg")
                }) =>
            {
                match crate::opus::OpusSource::open(File::open(path)?, offset) {
                    Err(symphonia::core::errors::Error::Unsupported(
                        "No Ogg Opus track was found",
                    )) => return Err(error.into()),
                    source => Box::new(source?),
                }
            }
            Err(error) => return Err(error.into()),
        };
        let duration = decoded.total_duration().unwrap_or_default();
        let player = Player::connect_new(&self.mixer);
        if paused {
            player.pause();
        }
        player.set_volume(self.volume);
        player.append(decoded);
        self.player = Some(player);
        self.duration = duration;
        self.offset = offset;
        Ok(())
    }
}

impl crate::session::AudioPlayback for Playback {
    fn play(&mut self, path: &Path) -> Result<(), crate::session::AudioError> {
        self.start(path, Duration::ZERO, false)?;
        self.source_path = Some(path.to_owned());
        Ok(())
    }

    fn reset(&mut self) {
        self.player = None;
        self.duration = Duration::ZERO;
        self.offset = Duration::ZERO;
        self.source_path = None;
    }

    fn is_finished(&self) -> bool {
        self.player.as_ref().is_some_and(Player::empty)
    }

    fn volume(&self) -> f32 {
        self.volume
    }

    fn set_volume(&mut self, value: f32) {
        self.volume = value;
        if let Some(player) = &self.player {
            player.set_volume(value);
        }
    }

    fn duration(&self) -> Duration {
        self.duration
    }

    fn position(&self) -> Duration {
        self.player
            .as_ref()
            .map_or(Duration::ZERO, |player| self.offset + player.get_pos())
    }

    fn is_paused(&self) -> bool {
        self.player.as_ref().is_none_or(Player::is_paused)
    }

    fn pause(&self) {
        if let Some(player) = &self.player {
            player.pause();
        }
    }

    fn resume(&self) {
        if let Some(player) = &self.player {
            player.play();
        }
    }

    fn seek(&mut self, target: Duration) -> Result<(), crate::session::AudioError> {
        if let Some(path) = self.source_path.clone() {
            let target = if self.duration.is_zero() {
                target
            } else {
                target.min(self.duration)
            };
            // A device callback may stop at EOF before consuming Player::try_seek.
            self.start(&path, target, self.is_paused())?;
        }
        Ok(())
    }
}
