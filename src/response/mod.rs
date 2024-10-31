use std::{collections::HashMap, io::{BufReader, Read}};

use builders::Builder;
use parse::parse_response;

use crate::HttpStream;

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
    pub fn status(&self) -> u16 { self.status }
    #[inline]
    pub fn content_length(&self) -> usize {
        match self.headers.get("Content-Length") {
            Some(l) => l.parse().unwrap_or(0),
            None => 0,
        }
    }
    /// Get the value of the given header key, if present
    #[inline]
    pub fn header(&self, key: &str) -> Option<&str> {
        self.headers.get(key).map(|s| s.as_str())
    }
    #[inline]
    pub fn version(&self) -> f32 { self.version }
    #[inline]
    pub fn headers(&self) -> &HashMap<String,String> { &self.headers }
    pub fn body(&mut self) -> &[u8] {
        let len = self.content_length();
        let mut buf:Vec<u8> = Vec::with_capacity(len);
        self.stream.read_to_end(&mut buf).unwrap();
        self.body = Some(buf);
        self.body.as_ref().unwrap()
    }
}
