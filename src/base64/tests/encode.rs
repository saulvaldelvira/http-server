use rb64::encode;

#[test]
fn hello_world() {
    let s = encode(b"Hello world!");
    assert_eq!(s, "SGVsbG8gd29ybGQh");
}

#[test]
fn two_remaining() {
    let s = encode(b"aaaaaaaaaaaaaaaaaaa");
    assert_eq!(s.capacity(), 28);
}

#[test]
fn one_remaining() {
    let s = encode(b"aaaaaaaaaaaaaaaaaaaa");
    assert_eq!(s.capacity(), 28);
}

#[test]
fn no_remaining() {
    let s = encode(b"aaaaaaaaaaaaaaaaaaaaa");
    assert_eq!(s.capacity(), 28);
}
