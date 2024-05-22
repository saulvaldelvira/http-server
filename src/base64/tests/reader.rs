use rb64::Base64Encoder;
use std::io::Read;

#[test]
fn reader() {
    let buf = "abcdefg";
    let expected = "YWJjZGVmZw==";
    let mut reader = Base64Encoder::new(buf.as_bytes());

    let mut out = [0_u8; 5];
    let mut i = 0;
    while let Ok(n) = reader.read(&mut out) {
        if n == 0 { return; }
        assert_eq!(&expected.as_bytes()[i..i+n], &out[0..n]);
        i += n;
    }
}
