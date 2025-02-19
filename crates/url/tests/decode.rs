use url::decode;

#[test]
fn valid() {
    assert_eq!("Hello world!", decode("Hello%20world!").unwrap());
}

#[test]
fn lorem_ipsum() {
    assert_eq!(
        "Lorém ipsuñ d0lér %20",
        decode("Lor%C3%A9m%20ipsu%C3%B1%20d0l%C3%A9r%20%2520").unwrap()
    );
}

#[test]
fn space() {
    assert_eq!("2 + 2 = 4", decode("2+%2B+2+%3D+4").unwrap());
}

#[test]
fn missing() {
    match decode("ABCD%") {
        Err(err) => assert_eq!(err.to_string(), "Missing byte after '%'"),
        _ => panic!(),
    }
    match decode("ABCD%1") {
        Err(err) => assert_eq!(err.to_string(), "Missing byte after '%'"),
        _ => panic!(),
    }
}

#[test]
fn invalid_utf8() {
    match decode("0123%99") {
        Err(err) => assert_eq!(
            err.to_string(),
            "invalid utf-8 sequence of 1 bytes from index 4"
        ),
        _ => panic!(),
    }
}
