pub mod handler;
pub mod encoding;
mod status;
pub use status::StatusCode;
mod method;
mod parse;
use parse::parse_request;
pub use method::RequestMethod;

use std::{collections::HashMap, env, ffi::OsStr, io::{BufReader, Read, Write}, net::TcpStream, path::Path};
use crate::Result;
use crate::request::encoding::Chunked;

/// HTTP Request
///
/// Represents an HTTP request
pub struct HttpRequest {
    method: RequestMethod,
    url: String,
    headers: HashMap<String,String>,
    params: HashMap<String,String>,
    response_headers: HashMap<String,String>,
    version: f32,
    stream: BufReader<TcpStream>,
    status: u16,
}

impl HttpRequest {
    /// Read and parse an HTTP request from the given [TcpStream]
    pub fn parse(stream: TcpStream) -> Result<Self>  {
        let stream = BufReader::new(stream);
        parse_request(stream)
    }
    #[inline]
    pub fn keep_alive(self) -> Result<Self> {
        let mut req = parse_request(self.stream)?;
        req.set_header("Connection", "keep-alive");
        Ok(req)
    }
    #[inline]
    pub fn stream(&self) -> &TcpStream { self.stream.get_ref() }
    /// Url of the request
    #[inline]
    pub fn url(&self) -> &str { &self.url }
    #[inline]
    pub fn set_url(&mut self, url: impl Into<String>) { self.url = url.into(); }
    /// Get the query parameters
    #[inline]
    pub fn params(&self) -> &HashMap<String,String> { &self.params }
    #[inline]
    pub fn param(&self, key: &str) -> Option<&str> { self.params.get(key).map(|s| s.as_str()) }
    /// Get the filename for the request
    ///
    /// It computes the path in the server corresponding to the
    /// request's url.
    ///
    pub fn filename(&self) -> Result<String> {
        let mut cwd = env::current_dir()?;
        cwd.push(
            Path::new(
                OsStr::new(&self.url[1..])
            )
        );
        let cwd = cwd.to_str().ok_or_else(|| "Error getting cwd")?;
        Ok(cwd.to_owned())
    }
    #[inline]
    pub fn method(&self) -> &RequestMethod { &self.method }
    #[inline]
    pub fn status(&self) -> u16 { self.status }
    #[inline]
    pub fn set_status(&mut self, status: u16) -> &mut Self {
        self.status = status;
        self
    }
    #[inline]
    pub fn version(&self) -> f32 { self.version }
    /// Get the value of the *Content-Length* HTTP header
    ///
    /// If the header is not present, or if it fails to parse
    /// it's value, it returns 0
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
    pub fn headers(&self) -> &HashMap<String,String> { &self.headers }
    #[inline]
    pub fn set_header(&mut self, key: impl Into<String>, value: impl Into<String>) {
        self.response_headers.insert(key.into(), value.into());
    }
    pub fn data(&mut self) -> Vec<u8> {
        let len = self.content_length();
        let mut buf:Vec<u8> = Vec::with_capacity(len);
        buf.resize(len, 0);
        self.stream.read_exact(&mut buf).unwrap();
        buf
    }
    pub fn read_data(&mut self, writer: &mut dyn Write) -> Result<()> {
        const CHUNK_SIZE:usize = 1024 * 1024;
        let mut buf:[u8;CHUNK_SIZE] = [0;CHUNK_SIZE];
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
    pub fn respond(&mut self) -> Result<()> {
        let response_line = format!("HTTP/{} {} {}\r\n", self.version, self.status, self.status_msg());
        self.stream.get_mut().write_all(response_line.as_bytes())?;
        let stream = self.stream.get_mut();
        for (k,v) in &self.response_headers {
           stream.write_all(k.as_bytes())?;
           stream.write_all(b": ")?;
           stream.write_all(v.as_bytes())?;
           stream.write_all(b"\r\n")?;
        }
        stream.write_all(b"\r\n")?;
        Ok(())
    }
    /// Respond to the request with the data of buf as a body
    pub fn respond_buf(&mut self, mut buf: &[u8]) -> Result<()> {
        self.set_header("Content-Length", buf.len().to_string());
        self.respond_reader(&mut buf)
    }
    /// Respond to the request with the given string
    #[inline]
    pub fn respond_str(&mut self, text: &str) -> Result<()> {
        self.respond_buf(text.as_bytes())
    }
    /// Respond to the request with the data read from reader as a body
    pub fn respond_reader(&mut self, reader: &mut dyn Read) -> Result<()> {
        self.respond()?;
        const CHUNK_SIZE: usize = 1024 * 1024;
        let mut buf: [u8; CHUNK_SIZE] = [0; CHUNK_SIZE];

        let stream = self.stream.get_mut();
        while let Ok(n) = reader.read(&mut buf) {
            if n == 0 { break; }
            stream.write_all(&buf[0..n])?;
        }
        Ok(())
    }
    /// Respond to the request as a chunked transfer
    ///
    /// This means that the Content-Length of the request doen't need to be known.
    pub fn respond_chunked(&mut self, reader: &mut dyn Read) -> Result<()> {
        self.set_header("Transfer-Encoding", "chunked");
        let mut reader = Chunked::new(reader);
        self.respond_reader(&mut reader)
    }
    /// Respond with a basic HTML error page
    #[inline]
    pub fn respond_error_page(&mut self) -> Result<()> {
        self.respond_str(&self.error_page())
    }
    /// Respond to the request with an 200 OK status
    #[inline]
    pub fn ok(&mut self) -> Result<()> {
        self.set_status(200).respond()
    }
    /// Respond to the request with an 403 FORBIDDEN status
    #[inline]
    pub fn forbidden(&mut self) -> Result<()> {
        self.set_status(403).respond_error_page()
    }
    /// Respond to the request with an 401 UNAUTHORIZED status
    #[inline]
    pub fn unauthorized(&mut self) -> Result<()> {
        self.set_status(401).respond_error_page()
    }
    /// Respond to the request with an 404 NOT FOUND status
    #[inline]
    pub fn not_found(&mut self) -> Result<()> {
        self.set_status(404).respond_error_page()
    }
    /// Respond to the request with an 500 INTERNAL SERVER ERROR status
    #[inline]
    pub fn server_error(&mut self) -> Result<()> {
        self.set_status(500).respond_error_page()
    }
    /// Returns a basic HTML error page of the given status
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
</html>")
    }
}

#[cfg(test)]
mod test {
    use std::str::FromStr;

    use crate::request::RequestMethod::{self,*};

    #[test]
    fn parse_method() {
        assert!(RequestMethod::from_str("unknown").is_err());
        let strs = vec!["GET","POST","PUT","DELETE"];
        let methods = vec![GET,POST,PUT,DELETE];
        let res:Vec<RequestMethod> =
            strs.iter()
            .map(|m| RequestMethod::from_str(m))
            .map(Result::unwrap).collect();
        assert_eq!(methods,res);
    }
}
