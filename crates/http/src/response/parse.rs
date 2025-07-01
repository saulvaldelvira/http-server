use std::{
    collections::HashMap,
    io::{BufRead, BufReader},
};

use super::HttpResponse;
use crate::{HttpStream, err};

pub(super) fn parse_response(
    mut stream: BufReader<Box<dyn HttpStream>>,
) -> crate::Result<HttpResponse> {
    let mut line = String::new();
    /* Parse request line */
    stream.read_line(&mut line)?;
    let mut space = line.split_whitespace().take(3);
    let version: f32 = space
        .next()
        .unwrap_or("")
        .replace("HTTP/", "")
        .parse()
        .or_else(|_| err!("Could not parse HTTP Version"))?;
    let status: u16 = space.next().unwrap_or("").parse()?;

    line.clear();
    /* Parse Headers */
    let mut headers = HashMap::new();
    while stream.read_line(&mut line).is_ok() {
        let l = line.trim();
        if l.is_empty() {
            break;
        }
        let mut splt = l.split(':');
        let key = splt.next().unwrap_or("").to_string();
        let value = splt.next().unwrap_or("").trim().to_string();
        headers.insert(key, value);
        line.clear();
    }
    Ok(HttpResponse {
        headers,
        version,
        stream,
        status,
        body: None,
    })
}
