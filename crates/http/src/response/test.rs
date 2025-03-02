use super::HttpResponse;

#[test]
fn response() {
    let mut res = HttpResponse::parse(
        r#"HTTP/1.1 200 OK
Date: 1-2-2024
Server: SRV
Content-Encoding: gzip
Content-Length: 20
Connection: Keep-Alive
Content-Type: text/plain

aaaaaaaaaaaaaaaaaaaa"#,
    )
    .expect("Expected response to parse successfully");

    res.read_body_into_buffer().unwrap();

    let expected = HttpResponse::builder()
        .version(1.1)
        .header("Date", "1-2-2024")
        .header("Server", "SRV")
        .header("Content-Length", "20")
        .header("Content-Encoding", "gzip")
        .header("Connection", "Keep-Alive")
        .header("Content-Type", "text/plain")
        .body(*b"aaaaaaaaaaaaaaaaaaaa")
        .build()
        .unwrap();

    assert_eq!(res, expected);
}
