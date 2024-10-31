use std::str::FromStr;

use crate::{request::RequestMethod::{self,*}, HttpRequest};

#[test]
fn parse_method() {
    assert!(RequestMethod::from_str("unknown").is_err());
    let strs = ["GET","POST","PUT","DELETE"];
    let methods = [GET,POST,PUT,DELETE];
    let res:Vec<RequestMethod> =
        strs.iter()
        .map(|m| RequestMethod::from_str(m))
        .map(Result::unwrap).collect();
    assert_eq!(methods,&res[..]);
    let m = RequestMethod::from_str("UNKNOWN");
    assert!(m.is_err_and(|err| err.get_message() == "Couldn't parse request method \"UNKNOWN\""));
}

#[test]
fn parse_test() {
    let parsed = HttpRequest::parse("GET / HTTP/1.0\r\nHEADER-TEST: Hello world!\r\n").unwrap();
    let expected = HttpRequest::builder()
        .url("/")
        .version(1.0)
        .header("HEADER-TEST", "Hello world!")
        .method(RequestMethod::GET)
        .build().unwrap();
    assert_eq!(expected, parsed);
}
