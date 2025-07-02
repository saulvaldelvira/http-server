use core::marker::PhantomData;
use std::{collections::HashMap, io::BufReader};

use crate::{HttpMethod, HttpRequest, stream};

pub struct Url;
pub struct NoUrl;

pub struct HttpRequestBuilder<U> {
    method: HttpMethod,
    url: Option<Box<str>>,
    headers: HashMap<Box<str>, Box<str>>,
    response_headers: HashMap<Box<str>, Box<str>>,
    params: HashMap<Box<str>, Box<str>>,
    version: f32,
    status: u16,
    body: Option<Box<[u8]>>,
    _pd: PhantomData<U>,
}

impl<U> HttpRequestBuilder<U> {
    pub fn method(mut self, m: HttpMethod) -> Self {
        self.method = m;
        self
    }

    pub fn header(mut self, k: impl Into<Box<str>>, v: impl Into<Box<str>>) -> Self {
        self.headers.insert(k.into(), v.into());
        self
    }

    pub fn response_header(mut self, k: impl Into<Box<str>>, v: impl Into<Box<str>>) -> Self {
        self.response_headers.insert(k.into(), v.into());
        self
    }

    pub fn param(mut self, k: impl Into<Box<str>>, v: impl Into<Box<str>>) -> Self {
        self.params.insert(k.into(), v.into());
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
}

impl HttpRequestBuilder<Url> {
    pub fn build(self) -> HttpRequest {
        HttpRequest {
            body: self.body,
            status: self.status,
            params: self.params,
            headers: self.headers,
            method: self.method,
            url: self.url.unwrap(),
            stream: BufReader::new(stream::dummy()),
            version: self.version,
            response_headers: self.response_headers,
        }
    }
}

impl HttpRequestBuilder<NoUrl> {
    pub fn new() -> Self {
        Self {
            method: HttpMethod::GET,
            url: None,
            response_headers: HashMap::new(),
            headers: HashMap::new(),
            params: HashMap::new(),
            status: 200,
            body: None,
            version: 1.0,
            _pd: PhantomData,
        }
    }

    pub fn url(self, url: impl Into<Box<str>>) -> HttpRequestBuilder<Url> {
        HttpRequestBuilder {
            url: Some(url.into()),
            body: self.body,
            status: self.status,
            params: self.params,
            headers: self.headers,
            version: self.version,
            response_headers: self.response_headers,
            method: self.method,
            _pd: PhantomData,
        }
    }
}

impl Default for HttpRequestBuilder<NoUrl> {
    fn default() -> Self {
        Self::new()
    }
}
