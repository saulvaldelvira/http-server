use http_srv::request::{HttpRequest, RequestMethod};

#[test]
fn request_test() {
    let parsed = HttpRequest::parse("GET / HTTP/1.0\r\nHEADER-TEST: Hello world!\r\n").unwrap();
    let expected = HttpRequest::builder()
                   .url("/")
                   .version(1.0)
                   .header("HEADER-TEST", "Hello world!")
                   .method(RequestMethod::GET)
                   .build().unwrap();
    assert_eq!(expected, parsed);
}