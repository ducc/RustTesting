#![allow(unused_imports)]
#![allow(dead_code)]

extern crate hyper;
extern crate byteorder;
extern crate base64;
extern crate serde;
#[macro_use] extern crate serde_json;

use std::io::{Read, Write, Cursor, BufWriter, BufReader, SeekFrom};
use std::io::prelude::*;
use std::fs::File;

use hyper::Client;
use hyper::client::{Body, Response};

use byteorder::{BigEndian, LittleEndian, WriteBytesExt, ReadBytesExt};

static URL: &'static str = "http://localhost:8080/tick";
const TRACK_INFO_VERSIONED: u8 = 1;
const TRACK_INFO_VERSION: u8 = 2;

fn main() {
    // debug
    //let enc_cursor = read_file("encoded.dab");
    //println!("main enc_cursor={:?}", enc_cursor.get_ref());
    /*for c in enc_cursor.get_ref() {
        print!("{}", *c as char);
    }
    println!();*/

    // decoding an encoded audio track
    //let mut enc_cursor = read_file("encoded.dab");
    //decode_track(&mut enc_cursor);

    // encoding an audio track
    /*encode_track(
        "Alan Walker - Fade [NCS Release]", // title
        "NCS", // author
        264175, // length
        "177671751", // identifier
        false, // stream
        "https://soundcloud.com/nocopyrightsounds/alan-walker-fade-ncs-release", // uri
        "soundcloud", // source
        0 // position
    );*/

    // sending an empty tick to remote node for statistics
    let mut buf = vec![];
    let _ = buf.write_i32::<BigEndian>(0);
    let mut cursor = post_tick(&buf, buf.len());
    decode_statistics(&mut cursor);

    let mut encoded = json_encode_track(
        "Alan Walker - Fade [NCS Release]", // title
        "NCS", // author
        264175, // length
        "177671751", // identifier
        false, // stream
        "https://soundcloud.com/nocopyrightsounds/alan-walker-fade-ncs-release", // uri
        "soundcloud", // source
        0 // position
    );
    println!("{:?}", encoded);
    let _ = encoded.write_i32::<BigEndian>(0);
    let mut cursor = post_tick(&encoded, encoded.len());
}

fn read_file(name: &str) -> Cursor<Vec<u8>> {
    let file = File::open(name).unwrap();
    let mut rdr = BufReader::new(file);
    let mut buf = vec![];
    let _ = rdr.read_to_end(&mut buf);
    Cursor::new(buf)
}

fn write_file(name: &str, content: &[u8]) {
    let file = File::create(name).unwrap();
    let mut buf = BufWriter::new(file);
    let _ = buf.write(base64::encode(content).as_bytes());
    buf.flush();
}

fn move_cursor(cursor: &mut Cursor<Vec<u8>>, steps: u64) {
    let pos = cursor.position();
    cursor.set_position(pos + steps);
}

fn write_string(cursor: &mut Cursor<Vec<u8>>, content: &str) {
    let _ = cursor.write_u8(content.len() as u8);
    let _ = cursor.write(content.as_bytes());
}

fn write_string_and_incr(cursor: &mut Cursor<Vec<u8>>, content: &str) {
    move_cursor(cursor, 1);
    let _ = cursor.write_u8(content.len() as u8);
    let _ = cursor.write(content.as_bytes());
}

fn read_string(cursor: &mut Cursor<Vec<u8>>) -> (String, usize) {
    let length = cursor.read_u8().unwrap() as usize;
    let mut b = vec![0; length];
    let _ = cursor.read_exact(&mut b);
    let c = b.clone();
    let s = String::from_utf8(b).unwrap();
    println!("\nread_string b={:?}\nread_string String::from_utf8={}\n", &c, &s);
    (s, length)
}

fn encode_track(title: &str, author: &str, length: i64, identifier: &str, stream: bool, uri: &str,
                source: &str, position: i64) {
    let buf: Vec<u8> = vec![];
    let mut cursor = Cursor::new(buf);

    let _ = cursor.write(&[0, 0]);
    let _ = cursor.write_u8(TRACK_INFO_VERSION);

    write_string_and_incr(&mut cursor, title);
    write_string_and_incr(&mut cursor, author);

    //0, 0, 0, 0, 0, 4, 7, -->239<--?
    let _ = cursor.write_i64::<BigEndian>(length);

    move_cursor(&mut cursor, 3);
    write_string(&mut cursor, identifier);

    move_cursor(&mut cursor, 1);
    if stream {
        let _ = cursor.write(&[0]);
    } else {
        let _ = cursor.write(&[1]);
    }

    write_string_and_incr(&mut cursor, uri);
    write_string_and_incr(&mut cursor, source);

    let _ = cursor.write_i64::<BigEndian>(position);

    println!("encode_track buf={:?}", cursor.get_ref());

    let mut new_buf: Vec<u8> = vec![];
    let _ = new_buf.write_i32::<BigEndian>(cursor.get_ref().len() as i32 | (TRACK_INFO_VERSIONED as i32) << 30);

    let _ = new_buf.write_all(cursor.get_ref());

    //println!("encode_track new_buf={:?}", new_buf);

    write_file("done.dab", &cursor.get_ref());
}

fn decode_track(cursor: &mut Cursor<Vec<u8>>) {
    let value = cursor.read_i32::<BigEndian>().unwrap();
    let m_flags = ((value as i64 & 0xC0000000) >> 30) as i32;
    let m_size = value & 0x3FFFFFFF;
    println!("decode_track value={} m_flags={} m_size={}", value, m_flags, m_size);

    /* position 1 and 2 might be part of java DataInputStream or BoundedInputStream, they are never
       used directly by lavaplayer */
    move_cursor(cursor, 2);
    let b = cursor.read_u8().unwrap();
    let version = match m_flags & (TRACK_INFO_VERSIONED as i32) {
        0 => 1,
        _ => b & 0xFF
    };
    println!("decode_track version={} b={:?}", version, b);

    // move past empty byte
    move_cursor(cursor, 1);
    let (title, length) = read_string(cursor);
    println!("decode_track title={} length={}", title, length);

    // 0 byte = EOF
    move_cursor(cursor, 1);
    let (author, length) = read_string(cursor);
    println!("decode_track length={} author={}", length, author);

    let duration = cursor.read_i64::<BigEndian>().unwrap();
    println!("decode_track length={}", length);

    // 3 bytes that might be java internal again
    move_cursor(cursor, 3);
    let (identifier, length) = read_string(cursor);
    println!("decode_track length={} identifier={}", length, identifier);

    // skip over EOF?
    move_cursor(cursor, 1);
    let mut buf = [0; 1];
    let _ = cursor.read_exact(&mut buf);
    let stream = match buf[0] {
        0 => true, _ => false
    };
    println!("decode_track stream={}", stream);

    // move past empty byte
    move_cursor(cursor, 1);
    let (uri, length) = read_string(cursor);
    println!("decode_track length={} uri={}", length, uri);

    // move past empty byte
    move_cursor(cursor, 1);
    let (source, length) = read_string(cursor);
    println!("decode_track length={} source={}", length, source);

    let position = cursor.read_i64::<BigEndian>().unwrap();
    println!("decode_track position={}", position);
}

fn post_tick(body: &[u8], size: usize) -> Cursor<Vec<u8>> {
    let mut resp = Client::new().post(URL).header(hyper::header::ContentType::json()).body(Body::BufBody(body, size)).send().unwrap();
    let mut body = vec![];
    let _ = resp.read_to_end(&mut body);
    Cursor::new(body)
}

fn decode_statistics(cursor: &mut Cursor<Vec<u8>>) {
    let size = cursor.read_i32::<BigEndian>().unwrap();
    let msg_type = cursor.read_u8().unwrap() & 0xFF;
    let version = cursor.read_u8().unwrap() & 0xFF;
    println!("decode_statistics size={} msg_type={} version={}", size, msg_type, version);

    // decoding node statistics message
    let playing_track_count = cursor.read_i32::<BigEndian>().unwrap();
    let total_track_count = cursor.read_i32::<BigEndian>().unwrap();
    let system_cpu_usage = cursor.read_f32::<BigEndian>().unwrap();
    let process_cpu_usage = cursor.read_f32::<BigEndian>().unwrap();
    println!("decode_statistics playing_track_count={} total_track_count={} system_cpu_usage={} \
    process_cpu_usage={}", playing_track_count, total_track_count, system_cpu_usage,
             process_cpu_usage);

}

fn json_encode_track(title: &str, author: &str, length: i64, identifier: &str, stream: bool,
                     uri: &str, source: &str, position: i64) -> Vec<u8> {
    let mut buf = vec![];

    let json = json!({
        "executor_id": 69i64,
        "title": title,
        "author": author,
        "length": length,
        "identifier": identifier,
        "is_stream": stream,
        "source": source,
        "volume": 100,
        "resampling_quality": "LOW",
        "opus_encoding_quality": 10,
        "channel_count": 0,
        "sample_rate": 0,
        "chunk_sample_count": 0,
        "codec": "",
        "position": position
    }).to_string();

    let _ = buf.write_i32::<BigEndian>(json.len() as i32); // msg length
    let _ = buf.write_i8(0); // type ordinal
    let _ = buf.write_i8(1); // codec version

    //let _ = buf.write_i8(0);
    let _ = buf.write_i32::<BigEndian>(json.len() as i32);
    let _ = buf.write(json.as_bytes());

    println!("{}", json.len());

    buf
}