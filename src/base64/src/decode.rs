use std::str::Chars;
use super::Result;

#[inline(always)]
fn next(chars: &mut Chars<'_>) -> Result<i8> {
    let c = chars.next().unwrap_or('=');
    let c = c as i8 - match c {
        'A'..='Z' => 'A' as i8,
        'a'..='z' => 'a' as i8 - 26,
        '0'..='9' => '0' as i8 - 52,
        '+' => '+' as i8 - 62,
        '/' => '/' as i8 - 63,
        '=' => c as i8,
        _ => return Err(format!("Unknown character to decode: {c}").into())
    };
    Ok(c)
}

/// Decode a Base64-encoded string
///
/// This function returns a [`Vec<u8>`] with the content of
/// the given string.
///
/// The text does not need to be padded with '='
///
/// # Example
/// ```
/// use rb64::decode;
///
/// let bytes = decode("SGVsbG8gd29ybGQh").unwrap();
/// let msg = String::from_utf8(bytes).unwrap();
/// assert_eq!(msg, "Hello world!");
/// ```
pub fn decode(text: &str) -> Result<Vec<u8>> {
    let capacity = text.len() as f64 / 4.0 * 3.0;
    let capacity = capacity.ceil() as usize;
    let mut decoded = Vec::<u8>::with_capacity(capacity);
    macro_rules! push {
        ($e:expr) => {
            decoded.push((($e) & 0b11111111) as u8);
        };
    }

    let mut chars = text.chars();
    'main: loop {
        let mut n = 0_u32;

        let mut offset = 4;
        for _ in 0..2 {
            let mut c = 0;

            for _ in 0..2 {
                c = next(&mut chars)?;
                n = n << 6 | c as u32;
            }

            if c == 0 { break 'main; }

            push!(n >> offset);
            offset *= 2;

        }
        push!(n);
    }
    Ok(decoded)
}
