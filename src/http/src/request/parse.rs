use std::{
    collections::HashMap,
    io::{BufRead, BufReader},
};

use crate::{err, request::HttpRequest, HttpStream, Result};

pub(super) fn parse_request(mut stream: BufReader<HttpStream>) -> Result<HttpRequest> {
    let mut line = String::new();
    /* Parse request line */
    stream.read_line(&mut line)?;
    let mut space = line.split_whitespace().take(3);
    let method = space.next().unwrap_or("").parse()?;
    let mut url = space.next().unwrap_or("");
    let mut params = HashMap::new();
    if url.contains('?') {
        /* Parse URL */
        let mut split = url.split('?');
        let new_url = split.next().unwrap_or("");
        let query = split.next().unwrap_or("");
        for arg in query.split('&') {
            let mut arg = arg.split('=');
            let k = arg.next().unwrap_or("");
            let v = arg.next().unwrap_or("");
            let k = url::decode(k)?.into();
            let v = url::decode(v)?.into();
            params.insert(k, v);
        }
        url = new_url;
    }
    let url = url::decode(url)?.into();
    let version: f32 = space
        .next()
        .unwrap_or("")
        .replace("HTTP/", "")
        .parse()
        .or_else(|_| err!("Could not parse HTTP Version"))?;
    line.clear();
    /* Parse Headers */
    let mut headers = HashMap::new();
    while stream.read_line(&mut line).is_ok() {
        if line == "\r\n" {
            break;
        }
        let mut splt = line.split(':');
        let key = splt.next().unwrap_or("").into();
        let value = splt
            .next()
            .unwrap_or("")
            .strip_prefix(' ')
            .unwrap_or("")
            .strip_suffix("\r\n")
            .unwrap_or("")
            .into();
        headers.insert(key, value);
        line.clear();
    }
    let response_headers = HashMap::new();
    Ok(HttpRequest {
        method,
        url,
        headers,
        params,
        response_headers,
        version,
        stream,
        status: 200,
        body: None,
    })
}
