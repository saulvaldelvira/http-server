use std::{collections::HashMap, io::BufReader};

use crate::{HttpResponse, stream};

pub struct HttpResponseBuilder {
    headers: HashMap<Box<str>, Box<str>>,
    status: u16,
    body: Option<Box<[u8]>>,
    version: f32,
}

impl HttpResponseBuilder {
    pub fn new() -> Self {
        Self {
            headers: HashMap::new(),
            status: 200,
            body: None,
            version: 1.0,
        }
    }

    pub fn header(mut self, k: impl Into<Box<str>>, v: impl Into<Box<str>>) -> Self {
        self.headers.insert(k.into(), v.into());
        self
    }

    pub fn version(mut self, v: f32) -> Self {
        self.version = v;
        self
    }

    pub fn status(mut self, code: u16) -> Self {
        self.status = code;
        self
    }

    pub fn body(mut self, body: impl Into<Box<[u8]>>) -> Self {
        self.body = Some(body.into());
        self
    }

    pub fn build(self) -> HttpResponse {
        HttpResponse {
            body: self.body,
            status: self.status,
            headers: self.headers,
            stream: BufReader::new(stream::dummy()),
            version: self.version,
        }
    }
}

impl Default for HttpResponseBuilder {
    fn default() -> Self {
        Self::new()
    }
}
