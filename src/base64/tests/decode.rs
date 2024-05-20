use rb64::decode;

#[test]
fn hello_world() {
    let decoded = decode("SGVsbG8gd29ybGQh").expect("Expected correct decoding");
    let decoded = String::from_utf8(decoded).expect("Expected valid UTF-8");
    assert_eq!(decoded, "Hello world!");
}

#[test]
fn padding() {
    let decoded = decode("VGhpcyBmcmFnbWVudCBoYXMgcGFkZGluZw==").expect("Expected correct decoding");
    let decoded = String::from_utf8(decoded).expect("Expected valid UTF-8");
    assert_eq!(decoded, "This fragment has padding");

    let decoded = decode("aGk=").expect("Expected correct decoding");
    let decoded = String::from_utf8(decoded).expect("Expected valid UTF-8");
    assert_eq!(decoded, "hi");
}

#[test]
fn invalid() {
    match decode("abcñ") {
        Err(err) => assert_eq!(err,"Unknown character to decode: 'ñ'"),
        Ok(_) => panic!("Expected Err")
    }
}
