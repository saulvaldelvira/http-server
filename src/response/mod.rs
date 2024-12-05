use std::{collections::HashMap, io::{self, BufReader, Read}};

use builders::Builder;
use parse::parse_response;

use crate::http::HttpStream;

mod parse;

#[derive(Builder)]
pub struct HttpResponse {
    #[builder(each = "header")]
    headers: HashMap<String,String>,
    #[builder(disabled = true)]
    #[builder(def = { BufReader::new(HttpStream::dummy()) })]
    stream: BufReader<HttpStream>,
    #[builder(def = 200u16)]
    status: u16,
    #[builder(optional = true)]
    body: Option<Vec<u8>>,
    #[builder(def = 1.0)]
    version: f32,
}

impl HttpResponse {
    pub fn parse(stream: impl Into<HttpStream>) -> crate::Result<Self>  {
        let stream = BufReader::new(stream.into());
        parse_response(stream)
    }
    #[inline]
    #[must_use]
    pub fn status(&self) -> u16 { self.status }

    #[inline]
    #[must_use]
    pub fn content_length(&self) -> usize {
        match self.headers.get("Content-Length") {
            Some(l) => l.parse().unwrap_or(0),
            None => 0,
        }
    }
    /// Get the value of the given header key, if present
    #[inline]
    #[must_use]
    pub fn header(&self, key: &str) -> Option<&str> {
        self.headers.get(key).map(String::as_str)
    }

    #[inline]
    #[must_use]
    pub fn version(&self) -> f32 { self.version }

    #[inline]
    #[must_use]
    pub fn headers(&self) -> &HashMap<String,String> { &self.headers }

    pub fn body(&mut self) -> Option<&[u8]> {
        let len = self.content_length();
        let mut buf:Vec<u8> = Vec::with_capacity(len);
        if self.stream.read_to_end(&mut buf).is_ok() {
            self.body = Some(buf);
        }
        self.body.as_deref()
    }
    pub fn write_to(&mut self, out: &mut dyn io::Write) -> io::Result<usize> {
        let mut buf = [0_u8; 1024];
        let mut total = 0;
        while let Ok(n) = self.stream.read(&mut buf) {
            out.write_all(&buf[0..n])?;
            total += n;
            if n == 0 { break }
        }
        out.flush()?;
        Ok(total)
    }
}
