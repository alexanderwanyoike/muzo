use std::{fs::File, num::NonZero, time::Duration};

use opus_decoder::OpusDecoder;
use rodio::{ChannelCount, SampleRate, Source};
use symphonia::core::{
    codecs::CODEC_TYPE_OPUS,
    errors::Error,
    formats::{FormatOptions, FormatReader, SeekMode, SeekTo},
    io::{MediaSourceStream, MediaSourceStreamOptions},
    meta::MetadataOptions,
    probe::Hint,
};

const RATE: SampleRate = NonZero::new(48_000).unwrap();
const MAX_FRAME_SAMPLES_PER_CHANNEL: usize = 5_760;
const SEEK_PREROLL: Duration = Duration::from_millis(80);
const OUTPUT_GAIN_BYTES: std::ops::Range<usize> = 16..18;
const GAIN_UNITS_PER_DB: f32 = 256.0;
const NANOS_PER_SECOND: u128 = Duration::from_secs(1).as_nanos();

pub struct OpusSource {
    format: Box<dyn FormatReader>,
    decoder: OpusDecoder,
    track_id: u32,
    channels: ChannelCount,
    duration: Option<Duration>,
    packet: Vec<f32>,
    cursor: usize,
    end: usize,
    discard_frames: u64,
    output_gain: f32,
}

impl OpusSource {
    pub fn open(file: File, position: Duration) -> Result<Self, Error> {
        let stream = MediaSourceStream::new(Box::new(file), MediaSourceStreamOptions::default());
        let options = FormatOptions {
            enable_gapless: true,
            ..Default::default()
        };
        let mut format = symphonia::default::get_probe()
            .format(
                Hint::new().with_extension("opus"),
                stream,
                &options,
                &MetadataOptions::default(),
            )?
            .format;
        let track = format
            .tracks()
            .iter()
            .find(|track| track.codec_params.codec == CODEC_TYPE_OPUS)
            .ok_or(Error::Unsupported("No Ogg Opus track was found"))?;
        let parameters = &track.codec_params;
        let track_id = track.id;
        let start_ts = parameters.start_ts;
        let channels = parameters
            .channels
            .and_then(|channels| ChannelCount::new(channels.count() as u16))
            .filter(|channels| channels.get() <= 2)
            .ok_or(Error::Unsupported("Opus channel mapping is unsupported"))?;
        let duration = parameters
            .time_base
            .zip(parameters.n_frames)
            .map(|(base, frames)| Duration::from(base.calc_time(frames)));
        let gain_db = parameters
            .extra_data
            .as_deref()
            .and_then(|header| header.get(OUTPUT_GAIN_BYTES))
            .and_then(|gain| gain.try_into().ok())
            .map_or(0.0, |gain| {
                f32::from(i16::from_le_bytes(gain)) / GAIN_UNITS_PER_DB
            });
        // RFC 7845 sections 4.6 and 5.1 specify preroll and a signed output gain.
        let preroll_start = position.saturating_sub(SEEK_PREROLL);
        let target_frame = pcm_frame(position);
        let target_ts = start_ts.saturating_add(target_frame);
        let discard_frames = if preroll_start.is_zero() {
            target_frame
        } else {
            let seek = format.seek(
                SeekMode::Accurate,
                SeekTo::TimeStamp {
                    ts: start_ts.saturating_add(pcm_frame(preroll_start)),
                    track_id,
                },
            )?;
            target_ts.saturating_sub(seek.actual_ts)
        };
        let decoder = OpusDecoder::new(RATE.get(), usize::from(channels.get()))
            .map_err(|_| Error::Unsupported("Could not initialize Opus decoding"))?;

        Ok(Self {
            format,
            decoder,
            track_id,
            channels,
            duration,
            packet: vec![0.0; MAX_FRAME_SAMPLES_PER_CHANNEL * usize::from(channels.get())],
            cursor: 0,
            end: 0,
            discard_frames,
            output_gain: 10.0_f32.powf(gain_db / 20.0),
        })
    }

    fn refill(&mut self) -> Option<()> {
        let channels = usize::from(self.channels.get());
        loop {
            let packet = self.format.next_packet().ok()?;
            if packet.track_id() != self.track_id {
                continue;
            }
            let Ok(frames) = self
                .decoder
                .decode_float(&packet.data, &mut self.packet, false)
            else {
                continue;
            };
            let start = frames.min(packet.trim_start() as usize);
            let end = frames.saturating_sub(packet.trim_end() as usize).max(start);
            let skip = self.discard_frames.min((end - start) as u64);
            self.discard_frames -= skip;
            self.cursor = (start + skip as usize) * channels;
            self.end = end * channels;
            if self.cursor < self.end {
                return Some(());
            }
        }
    }
}

fn pcm_frame(time: Duration) -> u64 {
    let rounded_frames =
        (time.as_nanos() * u128::from(RATE.get()) + NANOS_PER_SECOND / 2) / NANOS_PER_SECOND;
    rounded_frames.try_into().unwrap_or(u64::MAX)
}

impl Iterator for OpusSource {
    type Item = f32;

    fn next(&mut self) -> Option<Self::Item> {
        if self.cursor >= self.end {
            self.refill()?;
        }
        let sample = self.packet[self.cursor] * self.output_gain;
        self.cursor += 1;
        Some(sample)
    }
}

impl Source for OpusSource {
    fn current_span_len(&self) -> Option<usize> {
        None
    }
    fn channels(&self) -> ChannelCount {
        self.channels
    }
    fn sample_rate(&self) -> SampleRate {
        RATE
    }
    fn total_duration(&self) -> Option<Duration> {
        self.duration
    }
}
