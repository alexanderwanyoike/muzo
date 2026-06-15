use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader, Read, Seek, SeekFrom, Write};
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::thread;

use crate::domain::track::TrackFilePath;

#[derive(Clone)]
pub struct AudioStreamServer {
    addr: SocketAddr,
    routes: Arc<Mutex<HashMap<String, AudioRoute>>>,
}

#[derive(Clone)]
struct AudioRoute {
    file_path: PathBuf,
    mime_type: String,
}

impl AudioStreamServer {
    pub fn start() -> std::io::Result<Self> {
        let listener = TcpListener::bind(("127.0.0.1", 0))?;
        let addr = listener.local_addr()?;
        let routes = Arc::new(Mutex::new(HashMap::<String, AudioRoute>::new()));
        let thread_routes = Arc::clone(&routes);
        thread::spawn(move || {
            for stream in listener.incoming().flatten() {
                let routes = Arc::clone(&thread_routes);
                thread::spawn(move || {
                    let _ = handle_connection(stream, routes);
                });
            }
        });

        Ok(Self { addr, routes })
    }

    pub fn register(&self, file_path: &TrackFilePath, mime_type: &str) -> String {
        let token = ulid::Ulid::new().to_string();
        self.routes
            .lock()
            .expect("audio routes mutex poisoned")
            .insert(
                token.clone(),
                AudioRoute {
                    file_path: file_path.0.clone(),
                    mime_type: mime_type.to_string(),
                },
            );
        format!("http://{}/audio/{}", self.addr, token)
    }
}

fn handle_connection(
    mut stream: TcpStream,
    routes: Arc<Mutex<HashMap<String, AudioRoute>>>,
) -> std::io::Result<()> {
    let mut reader = BufReader::new(stream.try_clone()?);
    let mut request_line = String::new();
    reader.read_line(&mut request_line)?;
    let mut range_header = None;
    loop {
        let mut line = String::new();
        reader.read_line(&mut line)?;
        let trimmed = line.trim_end();
        if trimmed.is_empty() {
            break;
        }
        if let Some(value) = trimmed.strip_prefix("Range: ") {
            range_header = Some(value.to_string());
        }
    }

    let token = request_line
        .split_whitespace()
        .nth(1)
        .and_then(|path| path.strip_prefix("/audio/"));
    let Some(token) = token else {
        return write_response(&mut stream, "404 Not Found", "text/plain", b"not found");
    };
    let route = routes
        .lock()
        .expect("audio routes mutex poisoned")
        .get(token)
        .cloned();
    let Some(route) = route else {
        return write_response(&mut stream, "404 Not Found", "text/plain", b"not found");
    };

    write_file_response(&mut stream, route, range_header.as_deref())
}

fn write_file_response(
    stream: &mut TcpStream,
    route: AudioRoute,
    range_header: Option<&str>,
) -> std::io::Result<()> {
    let mut file = File::open(route.file_path)?;
    let file_len = file.metadata()?.len();
    let (status, start, end) = match range_header.and_then(|header| parse_range(header, file_len)) {
        Some((start, end)) => ("206 Partial Content", start, end),
        None => ("200 OK", 0, file_len.saturating_sub(1)),
    };
    let content_len = end.saturating_sub(start) + 1;
    let content_range = if status.starts_with("206") {
        format!("Content-Range: bytes {}-{}/{}\r\n", start, end, file_len)
    } else {
        String::new()
    };

    write!(
        stream,
        "HTTP/1.1 {}\r\nContent-Type: {}\r\nAccept-Ranges: bytes\r\n{}Content-Length: {}\r\nConnection: close\r\n\r\n",
        status, route.mime_type, content_range, content_len
    )?;
    file.seek(SeekFrom::Start(start))?;
    std::io::copy(&mut file.take(content_len), stream)?;
    Ok(())
}

fn parse_range(header: &str, file_len: u64) -> Option<(u64, u64)> {
    let range = header.strip_prefix("bytes=")?;
    let (start, end) = range.split_once('-')?;
    let start = start.parse::<u64>().ok()?;
    let end = if end.is_empty() {
        file_len.checked_sub(1)?
    } else {
        end.parse::<u64>().ok()?
    };
    if start > end || end >= file_len {
        return None;
    }
    Some((start, end))
}

fn write_response(
    stream: &mut TcpStream,
    status: &str,
    content_type: &str,
    body: &[u8],
) -> std::io::Result<()> {
    write!(
        stream,
        "HTTP/1.1 {}\r\nContent-Type: {}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
        status,
        content_type,
        body.len()
    )?;
    stream.write_all(body)
}

#[cfg(test)]
mod tests {
    use std::io::{Read, Write};
    use std::net::TcpStream;

    use super::*;

    #[test]
    fn it_serves_registered_audio_bytes_with_range_support() {
        let mut file = tempfile::NamedTempFile::new().unwrap();
        file.write_all(b"abcdef").unwrap();
        let server = AudioStreamServer::start().unwrap();
        let url = server.register(&TrackFilePath(file.path().to_path_buf()), "audio/mpeg");
        let addr = url
            .strip_prefix("http://")
            .and_then(|rest| rest.split_once('/'))
            .map(|(addr, _)| addr)
            .unwrap();
        let token_path = url.strip_prefix(&format!("http://{}", addr)).unwrap();

        let mut stream = TcpStream::connect(addr).unwrap();
        write!(
            stream,
            "GET {} HTTP/1.1\r\nHost: {}\r\nRange: bytes=1-3\r\n\r\n",
            token_path, addr
        )
        .unwrap();
        let mut response = String::new();
        stream.read_to_string(&mut response).unwrap();

        assert!(response.starts_with("HTTP/1.1 206 Partial Content"));
        assert!(response.contains("Content-Range: bytes 1-3/6"));
        assert!(response.ends_with("bcd"));
    }
}
