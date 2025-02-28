/* pub mod handler; */
use builders::Builder;
mod parse;
use std::{
    collections::HashMap,
    env,
    ffi::OsStr,
    io::{BufReader, Read, Write},
    path::Path,
};

use parse::parse_request;

use crate::{HttpMethod, HttpResponse, HttpStream, Result, StatusCode, encoding::Chunked};

/// HTTP Request
///
/// Represents an HTTP request
#[derive(Builder, Debug)]
pub struct HttpRequest {
    method: HttpMethod,
    url: Box<str>,
    #[builder(each = "header")]
    headers: HashMap<Box<str>, Box<str>>,
    #[builder(each = "param")]
    params: HashMap<Box<str>, Box<str>>,
    #[builder(each = "response_header")]
    response_headers: HashMap<Box<str>, Box<str>>,
    #[builder(def = 1.0)]
    version: f32,
    #[builder(disabled = true)]
    #[builder(def = { BufReader::new(HttpStream::dummy()) })]
    stream: BufReader<HttpStream>,
    #[builder(def = 200u16)]
    status: u16,
    #[builder(optional = true)]
    body: Option<Box<[u8]>>,
}

impl HttpRequest {
    /// Read and parse an HTTP request from the given [`HttpStream`]
    pub fn parse(stream: impl Into<HttpStream>) -> Result<Self> {
        let stream = BufReader::new(stream.into());
        parse_request(stream)
    }
    #[inline]
    pub fn keep_alive(self) -> Result<Self> {
        let mut req = parse_request(self.stream)?;
        req.set_header("Connection", "keep-alive");
        Ok(req)
    }
    #[inline]
    #[must_use]
    pub fn stream(&self) -> &HttpStream {
        self.stream.get_ref()
    }
    /// Url of the request
    #[inline]
    #[must_use]
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
    pub fn send_to(&self, mut stream: HttpStream) -> crate::Result<HttpResponse> {
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

    pub fn body(&mut self) -> Option<&[u8]> {
        let len = self.content_length().max(32);
        let mut buf = Vec::with_capacity(len);
        if self.stream.read_to_end(&mut buf).is_ok() {
            self.body = Some(buf.into_boxed_slice());
        }
        self.body.as_deref()
    }
    /// Reads the request body into [writer](Write)
    ///
    /// # Errors
    /// If, while reading of writing, some io Error is found
    pub fn read_body(&mut self, writer: &mut dyn Write) -> Result<()> {
        const CHUNK_SIZE: usize = 1024;
        let mut buf: [u8; CHUNK_SIZE] = [0; CHUNK_SIZE];
        let len = self.content_length();
        let n = len / CHUNK_SIZE;
        let remainder = len % CHUNK_SIZE;

        for _ in 0..n {
            self.stream.read_exact(&mut buf)?;
            writer.write_all(&buf)?;
        }

        if remainder > 0 {
            self.stream.read_exact(&mut buf[0..remainder])?;
            writer.write_all(&buf[0..remainder])?;
        }

        Ok(())
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
        let mut reader = Chunked::new(reader);
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
    }
}

#[cfg(test)]
mod test;
