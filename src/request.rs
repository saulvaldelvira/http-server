pub mod handler;

use std::{collections::HashMap, env, ffi::OsStr, fmt::Display, io::{BufRead, BufReader, Read, Write}, net::TcpStream, path::Path};
use crate::{Result,ServerError};


/// Request Method
///
/// Represents the method of the HTTP request
#[derive(Debug,Eq,Hash,PartialEq,Clone,Copy)]
pub enum RequestMethod {
    GET, POST, PUT, DELETE
}

impl RequestMethod {
    fn from_str(t: &str) -> Result<Self> {
        match t {
            "GET" => Ok(Self::GET),
            "POST" => Ok(Self::POST),
            "PUT" => Ok(Self::PUT),
            "DELETE" => Ok(Self::DELETE),
            _ => ServerError::from_str("Error parsing request method").err()
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
    response_headers: HashMap<String,String>,
    version: f32,
    stream: BufReader<TcpStream>,
    status: u16,
}

impl HttpRequest {
    /// Read and parse an HTTP request from the given [TcpStream]
    pub fn parse(stream: TcpStream) -> Result<Self>  {
        let mut stream = BufReader::new(stream);
        let mut line = String::new();

        /* Parse request line */
        stream.read_line(&mut line)?;
        let mut space = line.split_whitespace().take(3);
        let method = RequestMethod::from_str(space.next().unwrap_or(""))?;
        let url = space.next().unwrap().to_owned();
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
        Ok(Self { method, url, headers, response_headers, version, stream, status:200 })
    }
    /// Url of the request
    pub fn url(&self) -> &str { &self.url }
    /// Get the filename for the request
    ///
    /// It computes the path in the server corresponding to the
    /// request's url.
    ///
    pub fn filename(&self) -> Result<String> {
        let mut cwd = env::current_dir()?;
        cwd.push(
            Path::new(OsStr::new(
                &match self.url.as_str() {
                    "/" => "/index.html",
                    _ => &self.url,
                }[1..]
            ))
        );
        let cwd = cwd.to_str().ok_or_else(|| ServerError::from_str("Error getting cwd"))?;
        Ok(cwd.to_owned())
    }
    pub fn method(&self) -> &RequestMethod { &self.method }
    pub fn status(&self) -> u16 { self.status }
    pub fn set_status(&mut self, status: u16) -> &mut Self {
        self.status = status;
        self
    }
    /// Get a human-readable description of the request's status code
    pub fn status_msg(&self) -> &'static str {
        match self.status {
            200 => "OK",
            404 => "NOT FOUND",
            501 => "NOT IMPLEMENTED",
            403 => "FORBIDDEN",
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
    pub fn header(&self, key: &str) -> Option<&String> {
        self.headers.get(key)
    }
    pub fn set_header(&mut self, key: &str, value: &str) {
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
        let mut headers = String::new();
        for header in &self.response_headers {
           headers.push_str(header.0);
           headers.push_str(": ");
           headers.push_str(header.1);
           headers.push_str("\r\n");
        }
        headers += "\r\n";
        self.stream.get_mut().write_all(headers.as_bytes())?;
        Ok(())
    }
    /// Respond to the request with the data of buf as a body
    pub fn respond_buf(&mut self, buf: &[u8]) -> Result<()> {
        self.respond()?;
        self.stream.get_mut().write_all(&buf)?;
        Ok(())
    }

    /// Respond to the request with the data read from reader as a body
    pub fn respond_reader(&mut self, reader: &mut dyn Read) -> Result<()> {
        self.respond()?;
        const CHUNK_SIZE:usize = 1024 * 1024;
        let mut buf:[u8;CHUNK_SIZE] = [0;CHUNK_SIZE];

        while let Ok(n) = reader.read(&mut buf) {
            if n == 0 { break; }
            self.stream.get_mut().write_all(&buf[0..n])?;
        }
        Ok(())
    }
    /// Respond to the request with an 200 OK status
    pub fn ok(&mut self) -> Result<()> {
        self.set_status(200).respond()
    }
    /// Respond to the request with an 403 FORBIDDEN status
    pub fn forbidden(&mut self) -> Result<()> {
        self.send_error_page(403)
    }
    /// Respond to the request with an 404 NOT FOUND status
    pub fn not_found(&mut self) -> Result<()> {
        self.send_error_page(404)
    }
    /// Respond with a basic HTML error page of the given status
    pub fn send_error_page(&mut self, error: u16) -> Result<()> {
        self.status = error;
        let mut buf = self.error_page();
        self.respond_buf(&mut buf)
    }
    /// Returns a basic HTML error page of the given status
    pub fn error_page(&mut self) -> Vec<u8> {
        let code = self.status;
        let msg = self.status_msg();
        format!("<!DOCTYPE html>
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
