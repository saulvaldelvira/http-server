use std::borrow::Cow;

use crate::{from_hex_digit,Result};

/// UrlDecode the given string
pub fn decode(url: &str) -> Result<Cow<str>> {
    if !url.contains(['%','+']) {
        return Ok(url.into())
    }
    let mut result:Vec<u8> = Vec::new();
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
        let first = it.next().ok_or_else(|| "Missing byte after '%'".to_string())?;
        let second = it.next().ok_or_else(|| "Missing byte after '%'".to_string())?;
        let first = from_hex_digit(*first)?;
        let second = from_hex_digit(*second)?;
        let c = first << 4 | second;
        result.push(c);
    };
    let result = String::from_utf8(result).or_else(|err| Err(err.to_string()))?;
    Ok(result.into())
}
