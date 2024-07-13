use crate::Result;
use std::borrow::Cow;

/// UrlEncode the given string
pub fn encode(url: &str) -> Result<Cow<str>> {
    let is_ascii = |c: &u8|
                    matches!(c, b'0'..=b'9' | b'A'..=b'Z' |
                                b'a'..=b'z' | b'-' | b'.' |
                                b'_' | b'~');
    let len = url.as_bytes()
                 .iter()
                 .take_while(|c| is_ascii(c)).count();
    if len >= url.len() {
        return Ok(url.into());
    }
    let mut buf = String::new();
    for byte in url.as_bytes() {
        if is_ascii(byte) {
            buf.push(*byte as char);
        } else {
            buf.push('%');
            buf.push(to_hex_digit(byte >> 4));
            buf.push(to_hex_digit(byte & 0b1111));
        }
    }
    Ok(buf.into())
}

#[inline(always)]
fn to_hex_digit(digit: u8) -> char {
    (match digit {
        0..=9 => b'0' + digit,
        10..=255 => b'A' - 10 + digit,
    } as char)
}
