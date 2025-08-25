use alloc::{
    borrow::Cow,
    string::{String, ToString},
    vec::Vec,
};

use crate::Result;

/// UrlDecode the given string
pub fn decode(url: &str) -> Result<Cow<'_, str>> {
    if !url.contains(['%', '+']) {
        return Ok(url.into());
    }
    let mut result: Vec<u8> = Vec::new();
    let mut it = url.as_bytes().iter();
    while let Some(b) = it.next() {
        if *b == b'+' {
            result.push(b' ');
            continue;
        }
        if *b != b'%' {
            result.push(*b);
            continue;
        }
        let first = it.next().ok_or("Missing byte after '%'")?;
        let second = it.next().ok_or("Missing byte after '%'")?;
        let first = from_hex_digit(*first)?;
        let second = from_hex_digit(*second)?;
        let c = (first << 4) | second;
        result.push(c);
    }
    let result = String::from_utf8(result).map_err(|err| err.to_string())?;
    Ok(result.into())
}

#[inline(always)]
fn from_hex_digit(digit: u8) -> Result<u8> {
    match digit {
        b'0'..=b'9' => Ok(digit - b'0'),
        b'A'..=b'F' => Ok(digit - b'A' + 10),
        b'a'..=b'f' => Ok(digit - b'a' + 10),
        _ => Err(format!("{digit} is not a valid hex digit").into()),
    }
}
