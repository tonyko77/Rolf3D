//!  Various utilities

use std::{fs::File, io::Read};

#[inline]
pub fn buf_to_u16(buf: &[u8]) -> u16 {
    assert!(buf.len() >= 2);
    (buf[0] as u16) | ((buf[1] as u16) << 8)
}

#[inline]
pub fn buf_to_u32(buf: &[u8]) -> u32 {
    assert!(buf.len() >= 4);
    (buf[0] as u32) | ((buf[1] as u32) << 8) | ((buf[2] as u32) << 16) | ((buf[3] as u32) << 24)
}

#[inline]
pub fn buf_to_i32(buf: &[u8]) -> i32 {
    buf_to_u32(buf) as i32
}

#[inline]
pub fn buf_to_ascii(buf: &[u8], maxlen: usize) -> String {
    let mut len = 0;
    while len < maxlen && len < buf.len() && buf[len] >= 32 && buf[len] <= 127 {
        len += 1;
    }
    String::from(std::str::from_utf8(&buf[0..len]).unwrap())
}

/// Check if a file exists and can be read.
pub fn file_exist(filename: &str) -> bool {
    // open file
    let f = File::open(filename);
    if f.is_err() {
        return false;
    }
    // read a few bytes from the file
    let mut buf = [0; 4];
    let read_result = f.unwrap().read_exact(&mut buf);
    read_result.is_ok()
}

/// Read an entire binary file into a vector of bytes.
pub fn read_file_to_bytes(filename: &str, outbuf: &mut [u8]) -> Result<usize, String> {
    let mut f = File::open(&filename).map_err(|_| format!("File not found: {filename}"))?;
    let metadata = std::fs::metadata(&filename).map_err(|_| format!("Cannot read file metadata: {filename}"))?;
    let len = metadata.len() as usize;
    f.read(outbuf).map_err(|_| format!("Cannot read file: {filename}"))?;
    Ok(len)
}
