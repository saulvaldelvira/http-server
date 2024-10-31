use std::{collections::HashMap, io::{BufRead, BufReader}};

use crate::{server::error::err, HttpStream};

use super::HttpResponse;

pub (super) fn parse_response(mut stream: BufReader<HttpStream>) -> crate::Result<HttpResponse> {
    let mut line = String::new();
    /* Parse request line */
    stream.read_line(&mut line)?;
    let mut space = line.split_whitespace().take(3);
    let version: f32 = space.next().unwrap()
                        .replace("HTTP/", "")
                        .parse()
                        .or_else(|_| err!("Could not parse HTTP Version"))?;
    let status: u16 = space.next().unwrap().parse()?;

    line.clear();
    /* Parse Headers */
    let mut headers = HashMap::new();
    while stream.read_line(&mut line).is_ok() {
        if line == "\r\n" { break; }
        let mut splt = line.split(':');
        let key = splt.next().unwrap_or("").to_string();
        let value = splt.next().unwrap_or("")
                        .strip_prefix(' ').unwrap_or("")
                        .strip_suffix("\r\n").unwrap_or("")
                        .to_string();
        headers.insert(key, value);
        line.clear();
    }
    Ok(HttpResponse { headers, version, stream, status, body: None })
}

