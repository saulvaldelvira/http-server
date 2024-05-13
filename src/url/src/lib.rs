#[inline]
fn from_hex_digit(digit: u8) -> Result<u8> {
    match digit {
        b'0'..=b'9' => Ok(digit - b'0'),
        b'A'..=b'F' => Ok(digit - b'A' + 10),
        b'a'..=b'f' => Ok(digit - b'a' + 10),
        _ => Err(format!("{digit} is not a valid hex digit")),
    }
}

type Result<T> = std::result::Result<T,String>;

mod decode;
pub use decode::decode;

mod encode;
pub use encode::encode;