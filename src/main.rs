#![allow(unused_imports)]
#![allow(dead_code)]

extern crate hyper;
extern crate byteorder;
extern crate base64;

use std::io::{Read, Write, Cursor, BufWriter, BufReader, SeekFrom};
use std::io::prelude::*;
use std::fs::File;

use hyper::Client;
use hyper::client::{Body, Response};

use byteorder::{BigEndian, LittleEndian, WriteBytesExt, ReadBytesExt};

static URL: &'static str = "http://localhost:8080/tick";
const TRACK_INFO_VERSIONED: i32 = 1;

fn main() {
    // decoding an encoded audio track
    let mut enc_cursor = read_file("encoded.txt");
    decode_track(&mut enc_cursor);

    // sending an empty tick to remote node for statistics
    let mut buf = vec![];
    let _ = buf.write_i32::<BigEndian>(0);
    let mut cursor = post_tick(&buf, buf.len());
    decode_statistics(&mut cursor);
}

fn read_file(name: &str) -> Cursor<Vec<u8>> {
    let file = File::open(name).unwrap();
    let mut rdr = BufReader::new(file);
    let mut buf = vec![];
    let _ = rdr.read_to_end(&mut buf);
    Cursor::new(buf)
}

fn move_cursor(cursor: &mut Cursor<Vec<u8>>, steps: u64) {
    let pos = cursor.position();
    cursor.set_position(pos + steps);
}

fn read_string(cursor: &mut Cursor<Vec<u8>>) -> (String, usize) {
    let length = cursor.read_u8().unwrap() as usize;
    let mut b = vec![0; length];
    let _ = cursor.read_exact(&mut b);
    (String::from_utf8(b).unwrap(), length)
}

fn decode_track(cursor: &mut Cursor<Vec<u8>>) {
    let value = cursor.read_i32::<BigEndian>().unwrap();
    let m_flags = ((value as i64 & 0xC0000000) >> 30) as i32;
    let m_size = value & 0x3FFFFFFF;
    println!("decode_track m_flags={} m_size={}", m_flags, m_size);

    /* position 1 and 2 might be part of java DataInputStream or BoundedInputStream, they are never
       used directly by lavaplayer */
    move_cursor(cursor, 2);
    let mut buf = [0; 2];
    let _ = cursor.read_exact(&mut buf);
    let version = match m_flags & TRACK_INFO_VERSIONED {
        0 => 1,
        _ => buf[0] & 0xFF
    };
    println!("decode_track version={}", version);

    let (title, length) = read_string(cursor);
    println!("decode_track title={} length={}", title, length);

    // 0 byte = EOF
    move_cursor(cursor, 1);
    let (author, length) = read_string(cursor);
    println!("decode_track length={} author={}", length, author);

    let duration = cursor.read_i64::<BigEndian>().unwrap();
    println!("decode_track duration={}", duration);

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
}

fn post_tick(body: &[u8], size: usize) -> Cursor<Vec<u8>> {
    let mut resp = Client::new().post(URL).body(Body::BufBody(body, size)).send().unwrap();
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
