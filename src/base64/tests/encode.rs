use rb64::encode;

#[test]
fn hello_world() {
    let s = encode(b"Hello world!");
    assert_eq!(s, "SGVsbG8gd29ybGQh");
}
