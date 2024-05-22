use std::usize;

const TABLE: [char; 64] = [
    'A','B','C','D','E','F','G','H','I','J','K','L','M',
    'N','O','P','Q','R','S','T','U','V','W','X','Y','Z',
    'a','b','c','d','e','f','g','h','i','j','k','l','m',
    'n','o','p','q','r','s','t','u','v','w','x','y','z',
    '0','1','2','3','4','5','6','7','8','9','+','/'
];

/// Encode the given byte array to a Base 64 String
///
/// # Example
/// ```
/// use rb64::encode;
///
/// let enc = encode(b"Hello world!");
/// assert_eq!(enc, "SGVsbG8gd29ybGQh");
/// ```
pub fn encode(bytes: &[u8]) -> String {
    let capacity = bytes.len() as f64 / 3.0 * 4.0;
    let capacity = capacity.ceil() as usize + 2;
    let mut result = String::with_capacity(capacity);

    macro_rules! push {
        ($e:expr) => {
            if result.len() >= result.capacity() {
                /* The capacity will always be enough */
                unreachable!();
            }
            result.push(TABLE[($e & 0b111111) as usize]);
        };
    }

    let remaining = bytes.len() % 3;
    let len = bytes.len() - remaining;

    let mut i = 0;
    while i < len {
        let buf = &bytes[i..i+3];
        push!( buf[0] >> 2 );
        push!( buf[0] << 4 | buf[1] >> 4 );
        push!( buf[1] << 2 | buf[2] >> 6 );
        push!( buf[2] );
        i += 3;
    }

    if remaining > 0 {
        let buf = &bytes[i..];
        push!( buf[0] >> 2 );
        if remaining == 1 {
            push!( buf[0] << 4 );
            result.push_str("==");
            return result;
        }
        push!( buf[0] << 4 | buf[1] >> 4 );

        push!( buf[1] << 2 );
        result.push('=');
    }

    result
}
