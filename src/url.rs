use std::borrow::Cow;
use crate::{error::ServerError, Result};

#[inline]
fn from_hex_digit(digit: u8) -> Result<u8> {
    match digit {
        b'0'..=b'9' => Ok(digit - b'0'),
        b'A'..=b'F' => Ok(digit - b'A' + 10),
        b'a'..=b'f' => Ok(digit - b'a' + 10),
        _ => ServerError::from_string(format!("{digit} is not a valid hex digit")).err(),
    }
}

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
        let first = it.next().ok_or_else(|| ServerError::from_str("Missing byte after '%'"))?;
        let second = it.next().ok_or_else(|| ServerError::from_str("Missing byte after '%'"))?;
        let first = from_hex_digit(*first)?;
        let second = from_hex_digit(*second)?;
        let c = first << 4 | second;
        result.push(c);
    };
    return Ok(String::from_utf8(result)?.into());
}

#[cfg(test)]
mod test {
    use crate::url::decode;

    #[test]
    fn valid() {
        assert_eq!("Hello world!",
                   decode("Hello%20world!").unwrap());
    }

    #[test]
    fn lorem_ipsum() {
        assert_eq!("Lorém ipsuñ d0lér %20",
                   decode("Lor%C3%A9m%20ipsu%C3%B1%20d0l%C3%A9r%20%2520").unwrap());
    }

    #[test]
    fn space() {
        assert_eq!("2 + 2 = 4",
                   decode("2+%2B+2+%3D+4").unwrap());
    }

    #[test]
    fn missing() {
        match decode("ABCD%") {
            Err(err) => assert_eq!(err.to_string(), "Missing byte after '%'"),
            _ => panic!()
        }
        match decode("ABCD%1") {
            Err(err) => assert_eq!(err.to_string(), "Missing byte after '%'"),
            _ => panic!()
        }
    }

    #[test]
    fn invalid_utf8() {
        match decode("0123%99") {
            Err(err) => assert_eq!(err.to_string(),
                                   "invalid utf-8 sequence of 1 bytes from index 4"),
            _ => panic!()
        }
    }
}
