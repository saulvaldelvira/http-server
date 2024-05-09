pub mod handler;
pub mod encoding;

use std::{collections::HashMap, env, ffi::OsStr, fmt::Display, hash::Hash, io::{BufRead, BufReader, Read, Write}, net::TcpStream, path::Path, str::FromStr};
use crate::{url, Result, ServerError};
use crate::request::encoding::Chunked;

/// Request Method
///
/// Represents the method of the HTTP request
#[derive(Debug,Eq,Hash,PartialEq,Clone,Copy)]
pub enum RequestMethod {
    GET, POST, PUT, DELETE,
    HEAD, PATCH, CONNECT,
    OPTIONS, TRACE,
}

impl FromStr for RequestMethod {
    type Err = ServerError;
    fn from_str(t: &str) -> Result<Self> {
        match t {
            "GET" => Ok(Self::GET),
            "POST" => Ok(Self::POST),
            "PUT" => Ok(Self::PUT),
            "DELETE" => Ok(Self::DELETE),
            "HEAD" => Ok(Self::HEAD),
            "PATCH" => Ok(Self::PATCH),
            "CONNECT" => Ok(Self::CONNECT),
            "OPTIONS" => Ok(Self::OPTIONS),
            "TRACE" => Ok(Self::TRACE),
            _ => ServerError::from_string(format!("Couldn't parse request method ({t})")).err()
        }
    }
}

impl Display for RequestMethod {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

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

fn parse_request(mut stream: BufReader<TcpStream>) -> Result<HttpRequest> {
    let mut line = String::new();
    /* Parse request line */
    stream.read_line(&mut line)?;
    let mut space = line.split_whitespace().take(3);
    let method = space.next().unwrap_or("").parse()?;
    let mut url = space.next().unwrap();
    let mut params = HashMap::new();
    if url.contains("?") {
        /* Parse URL */
        let mut split = url.split("?");
        let new_url = split.next().unwrap();
        let query = split.next().unwrap_or("");
        for arg in query.split("&") {
            let mut arg = arg.split("=");
            let k = arg.next().unwrap_or("");
            let v = arg.next().unwrap_or("");
            let k = url::decode(k)?.into_owned();
            let v = url::decode(v)?.into_owned();
            params.insert(k, v);
        }
        url = new_url;
    }
    let url = url::decode(&url)?.into_owned();
    let version: f32 = space.next().unwrap()
                           .replace("HTTP/", "")
                           .parse()
                           .or_else(|_| ServerError::from_str("Could not parse HTTP Version").err())?;
    line.clear();
    /* Parse Headers */
    let mut headers = HashMap::new();
    while let Ok(_) = stream.read_line(&mut line) {
        if line == "\r\n" { break; }
        let mut splt = line.split(":");
        let key = splt.next().unwrap_or("").to_string();
        let value = splt.next().unwrap_or("")
                        .strip_prefix(" ").unwrap_or("")
                        .strip_suffix("\r\n").unwrap_or("")
                        .to_string();
        headers.insert(key, value);
        line.clear();
    }
    let response_headers = HashMap::new();
    Ok(HttpRequest { method, url, headers, params, response_headers, version, stream, status:200 })
}

impl HttpRequest {
    /// Read and parse an HTTP request from the given [TcpStream]
    pub fn parse(stream: TcpStream) -> Result<Self>  {
        let stream = BufReader::new(stream);
        parse_request(stream)
    }
    #[inline]
    pub fn keep_alive(self) -> Result<Self> {
        parse_request(self.stream)
    }
    #[inline]
    pub fn stream(&self) -> &TcpStream { self.stream.get_ref() }
    /// Url of the request
    #[inline]
    pub fn url(&self) -> &str { &self.url }
    #[inline]
    pub fn set_url(&mut self, url: String) { self.url = url; }
    /// Get the query parameters
    #[inline]
    pub fn params(&self) -> &HashMap<String,String> { &self.params }
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
        let cwd = cwd.to_str().ok_or_else(|| ServerError::from_str("Error getting cwd"))?;
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
    /// Get a human-readable description of the request's status code
    pub fn status_msg(&self) -> &'static str {
        match self.status {
            200 => "OK",
            201 => "CREATED",
            202 => "ACCEPTED",
            203 => "NON-AUTHORITATIVE INFORMATION",
            204 => "NO CONTENT",
            205 => "RESET CONTENT",
            206 => "PARTIAL CONTENT",
            300 => "MULTIPLE CHOICES",
            301 => "MOVED PERMANENTLY",
            302 => "FOUND",
            303 => "SEE OTHER",
            304 => "NOT MODIFIED",
            307 => "TEMPORARY REDIRECT",
            308 => "PERMANENT REDIRECT",
            400 => "BAD REQUEST",
            401 => "UNAUTHORIZED",
            403 => "FORBIDDEN",
            404 => "NOT FOUND",
            405 => "METHOD NOT ALLOWED",
            406 => "NOT ACCEPTABLE",
            407 => "PROXY AUTHENTICATION REQUIRED",
            408 => "REQUEST TIMEOUT",
            409 => "CONFLICT",
            501 => "NOT IMPLEMENTED",
            500 => "INTERNAL SERVER ERROR",
            _ => "?"
        }
    }
    /// Get the value of the *Content-Length* HTTP header
    ///
    /// If the header is not present, or if it fails to parse
    /// it's value, it returns 0
    pub fn content_length(&self) -> usize {
        match self.headers.get("Content-Length") {
            Some(l) => l.parse().unwrap_or(0),
            None => 0,
        }
    }
    /// Get the value of the given header key, if present
    #[inline]
    pub fn header(&self, key: &str) -> Option<&String> {
        self.headers.get(key)
    }
    #[inline]
    pub fn headers(&self) -> &HashMap<String,String> { &self.headers }
    #[inline]
    pub fn set_header<V: ToString>(&mut self, key: &str, value: V) {
        self.response_headers.insert(key.to_string(), value.to_string());
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
        self.set_header("Content-Length", buf.len());
        self.respond_reader(&mut buf)
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
    pub fn respond_error_page(&mut self) -> Result<()> {
        let mut buf = self.error_page();
        self.respond_buf(&mut buf)
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
    pub fn error_page(&mut self) -> Vec<u8> {
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
</html>").as_bytes().to_vec()
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
