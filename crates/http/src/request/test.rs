#![allow(clippy::unwrap_used)]

use std::str::FromStr;

use crate::{
    HttpRequest,
    request::HttpMethod::{self, *},
};

#[test]
fn parse_method() {
    assert!(HttpMethod::from_str("unknown").is_err());
    let strs = ["GET", "POST", "PUT", "DELETE"];
    let methods = [GET, POST, PUT, DELETE];
    let res: Vec<HttpMethod> = strs
        .iter()
        .map(|m| HttpMethod::from_str(m))
        .map(Result::unwrap)
        .collect();
    assert_eq!(methods, &res[..]);
    let m = HttpMethod::from_str("UNKNOWN");
    assert!(m.is_err_and(|err| err.get_message() == "Couldn't parse request method \"UNKNOWN\""));
}

#[test]
fn parse_test() {
    let parsed = HttpRequest::parse("GET / HTTP/1.0\r\nHEADER-TEST: Hello world!\r\n").unwrap();
    let expected = HttpRequest::builder()
        .url("/")
        .version(1.0)
        .header("HEADER-TEST", "Hello world!")
        .method(HttpMethod::GET)
        .build();
    assert_eq!(expected, parsed);
}

#[test]
fn send_to_test() {
    let req = HttpRequest::builder()
        .method(HttpMethod::GET)
        .url("/hello")
        .version(1.1)
        .body("BODY".as_bytes())
        .build();
    let mut b: Vec<u8> = Vec::new();
    req.write_to(&mut b).unwrap();

    let s = String::from_utf8(b).unwrap();
    assert_eq!(s, "GET /hello HTTP/1.1\r\nBODY\r\n");
}

#[test]
fn parse() {
    let req = "GET /index?param=abc&param2=def HTTP/1.0
HEADER1: header1

Hellooo";
    let mut req = HttpRequest::parse(req).unwrap();
    /* We need to force the request to read the body
    so the comparison bellow works*/
    req.read_body_into_buffer().unwrap();

    let expected = HttpRequest::builder()
        .method(HttpMethod::GET)
        .url("/index")
        .header("HEADER1", "header1")
        .param("param", "abc")
        .param("param2", "def")
        .body(*b"Hellooo")
        .build();

    assert_eq!(req, expected);
}
