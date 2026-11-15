use std::{
    fs::File,
    io::{BufReader, Seek},
    num::NonZero,
    time::Duration,
};

use ogg::reading::PacketReader;
use rodio::{ChannelCount, SampleRate, Source};
use ruopus::{OpusDecoder, ogg::OpusHead};
use symphonia::core::{
    codecs::CODEC_TYPE_OPUS,
    errors::Error,
    formats::{FormatOptions, FormatReader, SeekMode, SeekTo},
    io::{MediaSourceStream, MediaSourceStreamOptions},
    meta::MetadataOptions,
    probe::Hint,
};

const RATE: SampleRate = NonZero::new(48_000).unwrap();
const SEEK_PREROLL: Duration = Duration::from_millis(80);
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
    remaining_frames: Option<u64>,
    use_packet_trims: bool,
    output_gain: f32,
}

impl OpusSource {
    pub fn open(mut file: File, position: Duration) -> Result<Self, Error> {
        let single_page = single_audio_page(&mut file)?;
        file.rewind()?;
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
        let header = parameters
            .extra_data
            .as_deref()
            .and_then(|data| OpusHead::parse(data).ok())
            .ok_or(Error::DecodeError("Invalid Opus identification header"))?;
        let pre_skip = u64::from(header.pre_skip);
        let audible_frames = parameters
            .n_frames
            .map(|frames| frames.saturating_sub(pre_skip));
        let duration = audible_frames.map(|frames| {
            let rate = u64::from(RATE.get());
            Duration::new(
                frames / rate,
                ((frames % rate) * NANOS_PER_SECOND as u64 / rate) as u32,
            )
        });
        let gain_db = f32::from(header.output_gain_q8) / GAIN_UNITS_PER_DB;
        // Symphonia can report an end-trimmed first page as a start delay.
        let use_packet_trims = !single_page;
        // RFC 7845 sections 4.6 and 5.1 specify preroll and a signed output gain.
        let preroll_start = position.saturating_sub(SEEK_PREROLL);
        let target_frame = pre_skip.saturating_add(pcm_frame(position));
        let target_ts = start_ts.saturating_add(target_frame);
        let discard_frames = if preroll_start.is_zero() || !use_packet_trims {
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
        let decoder = OpusDecoder::new(usize::from(channels.get()));

        Ok(Self {
            format,
            decoder,
            track_id,
            channels,
            duration,
            packet: Vec::new(),
            cursor: 0,
            end: 0,
            discard_frames,
            remaining_frames: audible_frames
                .map(|frames| frames.saturating_sub(pcm_frame(position))),
            use_packet_trims,
            output_gain: 10.0_f32.powf(gain_db / 20.0),
        })
    }

    fn refill(&mut self) -> Option<()> {
        if self.remaining_frames == Some(0) {
            return None;
        }
        let channels = usize::from(self.channels.get());
        loop {
            let packet = self.format.next_packet().ok()?;
            if packet.track_id() != self.track_id {
                continue;
            }
            let Ok(mut samples) = self.decoder.decode_packet(&packet.data) else {
                continue;
            };
            if packet.data.len() == 1 {
                let frame_samples = usize::try_from(packet.block_dur())
                    .ok()?
                    .checked_mul(channels)?;
                samples.resize(frame_samples, 0.0);
            }
            self.packet = samples;
            let frames = self.packet.len() / channels;
            let (start, end) = if self.use_packet_trims {
                let start = frames.min(packet.trim_start() as usize);
                (
                    start,
                    frames.saturating_sub(packet.trim_end() as usize).max(start),
                )
            } else {
                (0, frames)
            };
            let skip = self.discard_frames.min((end - start) as u64);
            self.discard_frames -= skip;
            let start = start + skip as usize;
            let available = end - start;
            let emit = self.remaining_frames.map_or(available, |left| {
                available.min(usize::try_from(left).unwrap_or(usize::MAX))
            });
            if let Some(remaining) = &mut self.remaining_frames {
                *remaining -= emit as u64;
            }
            self.cursor = start * channels;
            self.end = (start + emit) * channels;
            if self.cursor < self.end {
                return Some(());
            }
        }
    }
}

fn single_audio_page(file: &mut File) -> Result<bool, Error> {
    let mut packets = PacketReader::new(BufReader::new(file));
    let mut opus_serial = None;
    while let Some(packet) = packets
        .read_packet()
        .map_err(|_| Error::DecodeError("Could not read the Ogg identification pages"))?
    {
        if packet.first_in_stream() && packet.data.starts_with(b"OpusHead") {
            opus_serial = Some(packet.stream_serial());
        } else if opus_serial == Some(packet.stream_serial())
            && !packet.data.starts_with(b"OpusTags")
            && packet.last_in_page()
        {
            return Ok(packet.last_in_stream());
        }
    }
    Ok(false)
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
