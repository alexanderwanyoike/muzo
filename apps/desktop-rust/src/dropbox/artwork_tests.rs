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
    let mut data = std::fs::read(path).unwrap();

    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let address = format!("http://{}", listener.local_addr().unwrap());
    let server = std::thread::spawn(move || {
        let (mut stream, _) = listener.accept().unwrap();
        let (start, end, headers) = range_request(&stream);
        assert_eq!(start, 0);
        assert!(end < FILE_LENGTH / 10);
        assert!(headers.contains("authorization: bearer fixture-token"));
        assert!(headers.contains(r#"dropbox-api-arg: {"path":"/music/track.mp3"}"#));
        data.resize(end + 1, 0);
        write!(
            stream,
            "HTTP/1.1 206 Partial Content\r\nContent-Length: {}\r\nContent-Range: bytes {start}-{end}/{FILE_LENGTH}\r\nConnection: close\r\n\r\n",
            data.len()
        )
        .unwrap();
        stream.write_all(&data).unwrap();
        stream.flush().unwrap();
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
