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

    bytes.chunks(3).for_each(|chunk| {
        for c in encode_chunk(chunk) {
            if result.len() >= result.capacity() {
                /* The capacity will always be enough */
                unreachable!();
            }
            result.push(c);
        }
    });

    result
}

pub (crate) fn encode_chunk(bytes: &[u8]) -> [char; 4] {
    let mut buf = ['='; 4];
    macro_rules! set {
        ($i:expr, $e:expr) => {
            buf[$i] = TABLE[($e & 0b111111) as usize];
        };
    }

    set!(0, bytes[0] >> 2 );
    if bytes.len() == 1 {
        set!(1, bytes[0] << 4);
        return buf;
    }
    set!(1, bytes[0] << 4 | bytes[1] >> 4 );
    if bytes.len() == 2 {
        set!(2, bytes[1] << 2);
        return buf;
    }
    set!(2, bytes[1] << 2 | bytes[2] >> 6 );
    set!(3, bytes[2] );
    buf
}
