use std::{
    io::{self, Read, Seek, SeekFrom},
    time::{Duration, Instant},
};

use lofty::{config::ParseOptions, probe::Probe};
use reqwest::{StatusCode, header::CONTENT_RANGE};

use crate::local_track::embedded_cover;

use super::{Client, DropboxError};

const RANGE_BYTES: u64 = 64 * 1024;
const ARTWORK_BUDGET_BYTES: u64 = 8 * 1024 * 1024;
const ARTWORK_TIMEOUT: Duration = Duration::from_secs(8);

impl Client {
    pub(crate) fn read_artwork(
        &self,
        token: &str,
        path: &str,
    ) -> Result<Option<Vec<u8>>, DropboxError> {
        let mut stream = ArtworkStream {
            client: self,
            token,
            path,
            length: 0,
            offset: 0,
            block_start: 0,
            block: Vec::new(),
            budget: ARTWORK_BUDGET_BYTES,
            deadline: Instant::now() + ARTWORK_TIMEOUT,
            authorization_failed: false,
        };
        let result = (|| {
            stream.fetch(0)?;
            let file = Probe::new(&mut stream)
                .options(ParseOptions::new().read_properties(false))
                .guess_file_type()?
                .read()
                .map_err(DropboxError::Artwork)?;
            Ok(embedded_cover(&file).map(|picture| picture.data().to_vec()))
        })();
        if stream.authorization_failed {
            return Err(DropboxError::Reconnect);
        }
        result
    }
}

struct ArtworkStream<'a> {
    client: &'a Client,
    token: &'a str,
    path: &'a str,
    length: u64,
    offset: u64,
    block_start: u64,
    block: Vec<u8>,
    budget: u64,
    deadline: Instant,
    authorization_failed: bool,
}

impl ArtworkStream<'_> {
    fn fetch(&mut self, start: u64) -> io::Result<()> {
        let allowance = RANGE_BYTES.min(self.budget);
        if allowance == 0 || self.deadline <= Instant::now() {
            return Err(io::Error::other("Dropbox artwork read budget exhausted"));
        }
        let end = start.saturating_add(allowance - 1);
        let response = self
            .client
            .download_range(
                self.token,
                self.path,
                start,
                end,
                self.deadline.saturating_duration_since(Instant::now()),
            )
            .map_err(|error| {
                self.authorization_failed |= error.status() == Some(StatusCode::UNAUTHORIZED);
                io::Error::other(error)
            })?;
        let range = response
            .headers()
            .get(CONTENT_RANGE)
            .and_then(|header| header.to_str().ok())
            .and_then(parse_content_range);
        let Some((first, last, length)) = range.filter(|(first, last, length)| {
            response.status() == StatusCode::PARTIAL_CONTENT
                && *first == start
                && *last >= *first
                && *last == end.min(length.saturating_sub(1))
                && (self.length == 0 || self.length == *length)
        }) else {
            return Err(io::Error::other(
                "Dropbox returned an invalid artwork range",
            ));
        };
        let count = last - first + 1;
        self.block.clear();
        response.take(count + 1).read_to_end(&mut self.block)?;
        if self.block.len() as u64 != count {
            return Err(io::Error::other(
                "Dropbox returned an incomplete artwork range",
            ));
        }
        self.length = length;
        self.block_start = first;
        self.budget -= count;
        Ok(())
    }
}

impl Read for ArtworkStream<'_> {
    fn read(&mut self, output: &mut [u8]) -> io::Result<usize> {
        if self.offset >= self.length || output.is_empty() {
            return Ok(0);
        }
        if self.offset < self.block_start
            || self.offset - self.block_start >= self.block.len() as u64
        {
            self.fetch(self.offset / RANGE_BYTES * RANGE_BYTES)?;
        }
        let available = &self.block[(self.offset - self.block_start) as usize..];
        let count = output.len().min(available.len());
        output[..count].copy_from_slice(&available[..count]);
        self.offset += count as u64;
        Ok(count)
    }
}

impl Seek for ArtworkStream<'_> {
    fn seek(&mut self, position: SeekFrom) -> io::Result<u64> {
        self.offset = match position {
            SeekFrom::Start(offset) => Some(offset),
            SeekFrom::End(offset) => self.length.checked_add_signed(offset),
            SeekFrom::Current(offset) => self.offset.checked_add_signed(offset),
        }
        .ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::InvalidInput,
                "Invalid Dropbox artwork offset",
            )
        })?;
        Ok(self.offset)
    }
}

fn parse_content_range(header: &str) -> Option<(u64, u64, u64)> {
    let (range, length) = header.strip_prefix("bytes ")?.split_once('/')?;
    let (first, last) = range.split_once('-')?;
    Some((
        first.parse().ok()?,
        last.parse().ok()?,
        length.parse().ok()?,
    ))
}
