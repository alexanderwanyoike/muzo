use std::{
    io::{BufRead, BufReader, Cursor, Write},
    net::TcpListener,
};

use image::{DynamicImage, ImageFormat, Rgba, RgbaImage};
use lofty::{config::WriteOptions, id3::v2::Id3v2Tag, picture::Picture, tag::TagExt};

use super::Client;

#[test]
fn it_reads_embedded_dropbox_artwork_before_playback_using_only_a_byte_range() {
    const FILE_LENGTH: usize = 24 * 1024 * 1024;
    let mut cover = Cursor::new(Vec::new());
    DynamicImage::ImageRgba8(RgbaImage::from_pixel(4, 4, Rgba([243, 40, 109, 255])))
        .write_to(&mut cover, ImageFormat::Png)
        .unwrap();
    let cover = cover.into_inner();
    let fixture = tempfile::tempdir().unwrap();
    let path = fixture.path().join("track.mp3");
    std::fs::write(&path, include_bytes!("../../tests/fixtures/tone.mp3")).unwrap();
    let mut tag = Id3v2Tag::new();
    tag.insert_picture(Picture::from_reader(&mut cover.as_slice()).unwrap());
    tag.save_to_path(&path, WriteOptions::default()).unwrap();
    let data = std::fs::read(path).unwrap();

    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let address = format!("http://{}", listener.local_addr().unwrap());
    let server = std::thread::spawn(move || {
        for expected_start in [0, FILE_LENGTH - 64 * 1024] {
            let (mut stream, _) = listener.accept().unwrap();
            let (start, end, headers) = range_request(&stream);
            assert_eq!(start, expected_start);
            assert!(end - start < FILE_LENGTH / 10);
            assert!(headers.contains("authorization: bearer fixture-token"));
            assert!(headers.contains(r#"dropbox-api-arg: {"path":"/music/track.mp3"}"#));
            let mut body = vec![0; end - start + 1];
            if start == 0 {
                body[..data.len()].copy_from_slice(&data);
            }
            write!(
                stream,
                "HTTP/1.1 206 Partial Content\r\nContent-Length: {}\r\nContent-Range: bytes {start}-{end}/{FILE_LENGTH}\r\nConnection: close\r\n\r\n",
                body.len()
            )
            .unwrap();
            stream.write_all(&body).unwrap();
        }
    });

    let client = Client::for_endpoint(&address).unwrap();
    let result = client.read_artwork("fixture-token", "/Music/Track.mp3");
    assert_eq!(result.unwrap().as_deref(), Some(cover.as_slice()));
    server.join().unwrap();
}

#[test]
fn it_rejects_an_unbounded_response_to_an_artwork_range_request() {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let address = format!("http://{}", listener.local_addr().unwrap());
    let server = std::thread::spawn(move || {
        let (mut stream, _) = listener.accept().unwrap();
        range_request(&stream);
        stream
            .write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 0\r\nConnection: close\r\n\r\n")
            .unwrap();
    });

    let client = Client::for_endpoint(&address).unwrap();
    assert!(
        client
            .read_artwork("fixture-token", "/Music/Track.mp3")
            .is_err()
    );
    server.join().unwrap();
}

#[test]
fn it_retries_m4a_artwork_when_a_range_fails_before_the_embedded_cover() {
    const RANGE: usize = 64 * 1024;

    let mut cover = Cursor::new(Vec::new());
    DynamicImage::ImageRgba8(RgbaImage::from_pixel(4, 4, Rgba([31, 85, 124, 255])))
        .write_to(&mut cover, ImageFormat::Png)
        .unwrap();
    let cover = cover.into_inner();
    let data = m4a_cover_in_second_range(&cover, RANGE);
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let address = format!("http://{}", listener.local_addr().unwrap());
    let server = std::thread::spawn(move || {
        for retry in [false, true] {
            for expected_start in [0, RANGE] {
                let (mut stream, _) = listener.accept().unwrap();
                let (start, end, _) = range_request(&stream);
                assert_eq!(start, expected_start);
                if !retry && start == RANGE {
                    stream.write_all(b"HTTP/1.1 503 Service Unavailable\r\nContent-Length: 0\r\nConnection: close\r\n\r\n").unwrap();
                    continue;
                }
                let end = end.min(data.len() - 1);
                write!(stream, "HTTP/1.1 206 Partial Content\r\nContent-Length: {}\r\nContent-Range: bytes {start}-{end}/{}\r\nConnection: close\r\n\r\n", end - start + 1, data.len()).unwrap();
                stream.write_all(&data[start..=end]).unwrap();
            }
        }
    });

    let client = Client::for_endpoint(&address).unwrap();
    let first = client.read_artwork("fixture-token", "/Music/Track.m4a");
    let second = client.read_artwork("fixture-token", "/Music/Track.m4a");
    server.join().unwrap();
    assert_eq!(second.unwrap().as_deref(), Some(cover.as_slice()));
    assert!(
        first.is_err(),
        "an interrupted M4A artwork range must remain retryable: {first:?}"
    );
}

fn m4a_cover_in_second_range(cover: &[u8], boundary: usize) -> Vec<u8> {
    fn atom(name: &[u8; 4], body: &[u8]) -> Vec<u8> {
        let len = u32::try_from(8 + body.len()).unwrap();
        [len.to_be_bytes().as_slice(), name, body].concat()
    }

    let ftyp = atom(b"ftyp", b"M4A \0\0\0\0isom");
    let data = atom(b"data", &[&[0, 0, 0, 14, 0, 0, 0, 0], cover].concat());
    let ilst = atom(b"ilst", &atom(b"covr", &data));
    let udta = atom(
        b"udta",
        &atom(b"meta", &[&[0; 4], ilst.as_slice()].concat()),
    );
    let padding = atom(b"free", &vec![0; boundary - ftyp.len() - 16]);
    [ftyp, atom(b"moov", &[padding, udta].concat())].concat()
}

#[test]
fn it_suppresses_retries_for_m4a_artwork_beyond_the_range_budget() {
    const RANGE: usize = 64 * 1024;
    const MAX_RANGES: usize = 128;

    let mut cover = Cursor::new(Vec::new());
    DynamicImage::ImageRgba8(RgbaImage::from_pixel(4, 4, Rgba([31, 85, 124, 255])))
        .write_to(&mut cover, ImageFormat::Png)
        .unwrap();
    let mut data = m4a_cover_in_second_range(&cover.into_inner(), RANGE);
    let tail = data.split_off(RANGE);
    let mut padding = vec![0; RANGE];
    padding[..4].copy_from_slice(&(RANGE as u32).to_be_bytes());
    padding[4..8].copy_from_slice(b"free");
    for _ in 1..MAX_RANGES {
        data.extend_from_slice(&padding);
    }
    data.extend_from_slice(&tail);
    let movie_header = u32::from_be_bytes(data[..4].try_into().unwrap()) as usize;
    let movie_length = u32::try_from(data.len() - movie_header).unwrap();
    data[movie_header..movie_header + 4].copy_from_slice(&movie_length.to_be_bytes());

    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    listener.set_nonblocking(true).unwrap();
    let address = format!("http://{}", listener.local_addr().unwrap());
    let server = std::thread::spawn(move || {
        use std::{
            io::ErrorKind,
            time::{Duration, Instant},
        };

        let deadline = Instant::now() + Duration::from_secs(10);
        let io_timeout = Some(Duration::from_secs(2));
        for index in 0..MAX_RANGES {
            let mut stream = loop {
                match listener.accept() {
                    Ok((stream, _)) => break stream,
                    Err(error) if error.kind() == ErrorKind::WouldBlock => {
                        assert!(Instant::now() < deadline, "received only {index} ranges");
                        std::thread::sleep(Duration::from_millis(1));
                    }
                    Err(error) => panic!("could not accept artwork range: {error}"),
                }
            };
            // An accepted socket can inherit the listener's nonblocking mode.
            stream.set_nonblocking(false).unwrap();
            stream.set_read_timeout(io_timeout).unwrap();
            stream.set_write_timeout(io_timeout).unwrap();
            let (start, end, _) = range_request(&stream);
            assert_eq!((start, end), (index * RANGE, (index + 1) * RANGE - 1));
            write!(stream, "HTTP/1.1 206 Partial Content\r\nContent-Length: {RANGE}\r\nContent-Range: bytes {start}-{end}/{}\r\nConnection: close\r\n\r\n", data.len()).unwrap();
            stream.write_all(&data[start..=end]).unwrap();
        }
    });
    let client = Client::for_endpoint(&address).unwrap();
    let result = client.read_artwork("fixture-token", "/Music/Track.m4a");
    server.join().unwrap();
    assert!(
        matches!(result, Ok(None)),
        "artwork beyond the per-track budget should not be retried: {result:?}"
    );
}

fn range_request(stream: &std::net::TcpStream) -> (usize, usize, String) {
    let mut request = BufReader::new(stream);
    let mut route = String::new();
    request.read_line(&mut route).unwrap();
    assert!(route.starts_with("GET /2/files/download "));
    let mut headers = String::new();
    loop {
        let mut line = String::new();
        request.read_line(&mut line).unwrap();
        if line == "\r\n" || line.is_empty() {
            break;
        }
        headers.push_str(&line.to_ascii_lowercase());
    }
    let range = headers
        .lines()
        .find_map(|line| line.strip_prefix("range: bytes="))
        .unwrap();
    let (start, end) = range.split_once('-').unwrap();
    (start.parse().unwrap(), end.parse().unwrap(), headers)
}

#[test]
fn it_requires_reauthorization_when_dropbox_rejects_an_artwork_token() {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let address = format!("http://{}", listener.local_addr().unwrap());
    let server = std::thread::spawn(move || {
        let (mut stream, _) = listener.accept().unwrap();
        range_request(&stream);
        stream
            .write_all(
                b"HTTP/1.1 401 Unauthorized\r\nContent-Length: 0\r\nConnection: close\r\n\r\n",
            )
            .unwrap();
    });
    let client = Client::for_endpoint(&address).unwrap();
    let result = client.read_artwork("expired-fixture-token", "/Music/Track.mp3");
    assert!(
        matches!(result, Err(super::DropboxError::Reconnect)),
        "expected a new authorization after Dropbox rejected the token: {result:?}"
    );
    server.join().unwrap();
}
