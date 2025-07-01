/* pub mod handler; */
use builders::Builder;
mod parse;
use core::fmt;
use std::{
    collections::HashMap,
    env,
    ffi::OsStr,
    io::{BufRead, BufReader, Read, Write},
    path::Path,
};

use parse::parse_request;

use crate::{
    HttpMethod, HttpResponse, HttpStream, Result, StatusCode,
    encoding::Chunked,
    stream::{self, IntoHttpStream},
};

/// HTTP Request
///
/// Represents an HTTP request
#[derive(Builder)]
pub struct HttpRequest {
    #[builder(def = { HttpMethod::GET })]
    method: HttpMethod,
    url: Box<str>,
    #[builder(map = "header")]
    headers: HashMap<Box<str>, Box<str>>,
    #[builder(map = "param")]
    params: HashMap<Box<str>, Box<str>>,
    #[builder(map = "response_header")]
    response_headers: HashMap<Box<str>, Box<str>>,
    #[builder(def = 1.0)]
    version: f32,
    #[builder(def = { BufReader::new(stream::dummy())} )]
    stream: BufReader<Box<dyn HttpStream>>,
    #[builder(def = 200u16)]
    status: u16,
    #[builder(optional = true)]
    body: Option<Box<[u8]>>,
}

impl fmt::Debug for HttpRequest {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("HttpRequest")
            .field("method", &self.method)
            .field("url", &self.url)
            .field("headers", &self.headers)
            .field("params", &self.params)
            .field("response_headers", &self.response_headers)
            .field("version", &self.version)
            .field("status", &self.status)
            .field("body", &self.body)
            .finish()
    }
}

impl HttpRequest {
    /// Read and parse an HTTP request from the given [`HttpStream`]
    pub fn parse<S: IntoHttpStream>(stream: S) -> Result<Self> {
        let stream: Box<dyn HttpStream> = Box::new(stream.into_http_stream());
        parse_request(BufReader::new(stream))
    }

    #[inline]
    pub fn keep_alive(self) -> Result<Self> {
        let mut req = parse_request(self.stream)?;
        req.set_header("Connection", "keep-alive");
        Ok(req)
    }

    #[inline]
    pub fn stream(&self) -> &BufReader<Box<dyn HttpStream>> {
        &self.stream
    }

    #[inline]
    pub fn stream_mut(&mut self) -> &mut BufReader<Box<dyn HttpStream>> {
        &mut self.stream
    }

    /// Url of the request
    #[inline]
    pub fn url(&self) -> &str {
        &self.url
    }

    #[inline]
    pub fn set_url(&mut self, url: impl Into<Box<str>>) {
        self.url = url.into();
    }
    /// Get the query parameters
    #[inline]
    #[must_use]
    pub fn params(&self) -> &HashMap<Box<str>, Box<str>> {
        &self.params
    }

    #[inline]
    #[must_use]
    pub fn param(&self, key: &str) -> Option<&str> {
        self.params.get(key).map(AsRef::as_ref)
    }

    /// Get the filename for the request
    ///
    /// It computes the path in the server corresponding to the
    /// request's url.
    ///
    pub fn filename(&self) -> Result<Box<str>> {
        let mut cwd = env::current_dir()?;
        cwd.push(Path::new(OsStr::new(&self.url[1..])));
        let cwd = cwd.to_str().ok_or("Error getting cwd")?;
        Ok(Box::from(cwd))
    }

    /// Writes the request into the given [Write] object.
    pub fn write_to(&self, f: &mut dyn Write) -> Result<()> {
        write!(f, "{} {}", self.method(), self.url())?;
        if !self.params().is_empty() {
            write!(f, "?")?;
            for (k, v) in self.params() {
                let ke = url::encode(k).unwrap_or("".into());
                let ve = url::encode(v).unwrap_or("".into());
                write!(f, "{ke}={ve}&")?;
            }
        }
        write!(f, " HTTP/{}\r\n", self.version())?;

        for (k, v) in self.headers() {
            write!(f, "{k}: {v}\r\n")?;
        }

        if let Some(ref b) = self.body {
            f.write_all(b)?;
        }

        write!(f, "\r\n")?;

        Ok(())
    }

    /// Sends the ``HttpRequest`` to a [stream](HttpStream)
    ///
    /// # Errors
    /// If the transfer fails, returns the error
    pub fn send_to<Out: IntoHttpStream>(&self, stream: Out) -> crate::Result<HttpResponse> {
        let mut stream = stream.into_http_stream();
        self.write_to(&mut stream)?;
        stream.flush()?;
        HttpResponse::parse(stream)
    }
    #[inline]
    #[must_use]
    pub fn method(&self) -> &HttpMethod {
        &self.method
    }

    #[inline]
    #[must_use]
    pub fn status(&self) -> u16 {
        self.status
    }

    #[inline]
    pub fn set_status(&mut self, status: u16) -> &mut Self {
        self.status = status;
        self
    }

    #[inline]
    #[must_use]
    pub fn version(&self) -> f32 {
        self.version
    }

    /// Get the value of the *Content-Length* HTTP header
    ///
    /// If the header is not present, or if it fails to parse
    /// it's value, it returns 0
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
        self.headers.get(key).map(AsRef::as_ref)
    }

    #[inline]
    #[must_use]
    pub fn headers(&self) -> &HashMap<Box<str>, Box<str>> {
        &self.headers
    }

    #[inline]
    pub fn set_header(&mut self, key: impl Into<Box<str>>, value: impl Into<Box<str>>) {
        self.response_headers.insert(key.into(), value.into());
    }

    /// Reads the body from the stream into the buffer.
    ///
    /// This method is primarly used by [`body`](Self::body), and
    /// for unit tests, where we need to force-load the body into
    /// the stream's buffer.
    pub(crate) fn read_body_into_buffer(&mut self) -> Result<()> {
        let len = self.content_length();
        let mut buf = Vec::with_capacity(len);
        self.stream.read_to_end(&mut buf)?;
        self.body = Some(buf.into_boxed_slice());
        Ok(())
    }

    /// Reads the body from the [`stream`] into the request's buffer.
    ///
    /// # NOTE
    /// This loads the whole body of the request into memory,
    /// and it'll stick with the request for it's lifetime.
    /// It's not very efficient memory-wise for requests with big bodies.
    ///
    /// For a nicer way to process a request's body, see the
    /// [read_body](Self::read_body) function.
    ///
    /// # Errors
    /// If some IO error happens when reading the body from the [`stream`]
    ///
    /// # Returns
    /// And option of &[u8]. A None variant means the request doesn't have a body.
    /// For example, GET request don't usually have a body.
    ///
    /// [`stream`]: HttpStream
    pub fn body(&mut self) -> Result<Option<&[u8]>> {
        if self.body.is_none() {
            self.read_body_into_buffer()?;
        }
        Ok(self.body.as_deref())
    }

    /// Returns true if the [`stream`](HttpStream) has a body,
    /// and false if it's empty.
    ///
    /// This method is preferred to check the presence of a body,
    /// over calling [body](Self::body) and checking the returned Option,
    /// since this function doesn't allocate memory, nor mutates the request.
    ///
    /// # Errors
    /// If some IO error happens in the process of checking the
    /// [`stream`]'s availability
    ///
    /// [`stream`]: HttpStream
    pub fn has_body(&mut self) -> Result<bool> {
        Ok(self.body.is_some() || !self.stream.fill_buf()?.is_empty())
    }

    /// Reads the request body into [writer](Write)
    ///
    /// # Errors
    /// If, while reading or writing, some io Error is found
    pub fn read_body(&mut self, out: &mut dyn Write) -> Result<usize> {
        let mut total = 0;
        loop {
            let slice = self.stream.fill_buf()?;
            if slice.is_empty() {
                break;
            }
            out.write_all(slice)?;

            let len = slice.len();
            self.stream.consume(len);
            total += len;
        }
        out.flush()?;
        Ok(total)
    }
    /// Respond to the request without a body
    ///
    /// # Errors
    /// If some io error is produced while sending the request
    pub fn respond(&mut self) -> Result<()> {
        let response_line = format!(
            "HTTP/{} {} {}\r\n",
            self.version,
            self.status,
            self.status_msg()
        );
        self.stream.get_mut().write_all(response_line.as_bytes())?;
        let stream = self.stream.get_mut();
        for (k, v) in &self.response_headers {
            stream.write_all(k.as_bytes())?;
            stream.write_all(b": ")?;
            stream.write_all(v.as_bytes())?;
            stream.write_all(b"\r\n")?;
        }
        stream.write_all(b"\r\n")?;
        Ok(())
    }
    /// Respond to the request with the data of buf as a body
    ///
    /// # Errors
    /// If some io error is produced while sending the request
    pub fn respond_buf(&mut self, mut buf: &[u8]) -> Result<()> {
        self.set_header("Content-Length", buf.len().to_string());
        self.respond_reader(&mut buf)
    }
    /// Respond to the request with the given string
    ///
    /// # Errors
    /// If some io error is produced while sending the request
    #[inline]
    pub fn respond_str(&mut self, text: &str) -> Result<()> {
        self.respond_buf(text.as_bytes())
    }
    /// Respond to the request with the data read from reader as a body
    ///
    /// # Errors
    /// If some io error is produced while sending the request
    pub fn respond_reader(&mut self, reader: &mut dyn Read) -> Result<()> {
        const CHUNK_SIZE: usize = 1024;
        let mut buf: [u8; CHUNK_SIZE] = [0; CHUNK_SIZE];

        self.respond()?;

        let stream = self.stream.get_mut();
        while let Ok(n) = reader.read(&mut buf) {
            if n == 0 {
                break;
            }
            stream.write_all(&buf[0..n])?;
        }
        Ok(())
    }
    /// Respond to the request as a chunked transfer
    ///
    /// This means that the Content-Length of the request doen't need to be known.
    ///
    /// # Errors
    /// If some io error is produced while sending the request
    pub fn respond_chunked(&mut self, reader: &mut dyn Read) -> Result<()> {
        self.set_header("Transfer-Encoding", "chunked");
        let mut reader = Chunked::with_default_size(reader);
        self.respond_reader(&mut reader)
    }
    /// Respond with a basic HTML error page
    ///
    /// # Errors
    /// If some io error is produced while sending the request
    #[inline]
    pub fn respond_error_page(&mut self) -> Result<()> {
        self.set_header("Content-Type", "text/html");
        self.respond_str(&self.error_page())
    }
    /// Respond to the request with an 200 OK status
    ///
    /// # Errors
    /// If some io error is produced while sending the request
    #[inline]
    pub fn ok(&mut self) -> Result<()> {
        self.set_status(200).respond()
    }
    /// Respond to the request with an 403 FORBIDDEN status
    ///
    /// # Errors
    /// If some io error is produced while sending the request
    #[inline]
    pub fn forbidden(&mut self) -> Result<()> {
        self.set_status(403).respond_error_page()
    }
    /// Respond to the request with an 401 UNAUTHORIZED status
    ///
    /// # Errors
    /// If some io error is produced while sending the request
    #[inline]
    pub fn unauthorized(&mut self) -> Result<()> {
        self.set_status(401).respond_error_page()
    }
    /// Respond to the request with an 404 NOT FOUND status
    ///
    /// # Errors
    /// If some io error is produced while sending the request
    #[inline]
    pub fn not_found(&mut self) -> Result<()> {
        self.set_status(404).respond_error_page()
    }
    /// Respond to the request with an 500 INTERNAL SERVER ERROR status
    ///
    /// # Errors
    /// If some io error is produced while sending the request
    #[inline]
    pub fn server_error(&mut self) -> Result<()> {
        self.set_status(500).respond_error_page()
    }
    #[inline]
    #[must_use]
    pub fn is_http_ok(&self) -> bool {
        self.status.is_http_ok()
    }
    #[inline]
    #[must_use]
    pub fn is_http_err(&self) -> bool {
        self.status.is_http_err()
    }
    #[inline]
    #[must_use]
    pub fn status_msg(&self) -> &'static str {
        self.status.status_msg()
    }
    /// Returns a basic HTML error page of the given status
    #[must_use]
    pub fn error_page(&self) -> String {
        let code = self.status;
        let msg = self.status_msg();
        format!(
            "<!DOCTYPE html>
<html lang=\"en\">
    <head>
        <meta charset=\"utf-8\">
        <title>{code} {msg}</title>
    </head>
<body>
    <h1>{code} {msg}</h1>
</body>
</html>"
        )
    }
}

impl PartialEq for HttpRequest {
    fn eq(&self, other: &Self) -> bool {
        self.method == other.method
            && self.url == other.url
            && self.headers == other.headers
            && self.params == other.params
            && self.response_headers == other.response_headers
            && self.version == other.version
            && self.status == other.status
            && self.body == other.body
    }
}

#[cfg(test)]
mod test;
