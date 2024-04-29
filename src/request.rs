pub mod handler;

use std::{collections::HashMap, env, ffi::OsStr, fmt::Display, io::{BufRead, BufReader, Read, Write}, net::TcpStream, path::Path};
use crate::{Result,ServerError};


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

pub struct HttpRequest {
    method: RequestMethod,
    url: String,
    headers: HashMap<String,String>,
    data: Vec<u8>,
    version: f32,
    stream: TcpStream,
    status: u16,
}

impl HttpRequest {
    pub fn parse(stream: TcpStream) -> Result<Self>  {
        let mut reader = BufReader::new(&stream);
        let mut line = String::new();

        /* Parse request line */
        reader.read_line(&mut line)?;
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
        while let Ok(_) = reader.read_line(&mut line) {
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

        /* Get body (if present) */
        let len =
            match headers.get("Content-Length") {
                Some(l) => l.parse().unwrap_or(0),
                None => 0,
            };
        let mut data:Vec<u8> = Vec::new();
        if len > 0 {
            data.resize(len, 0);
            reader.read_exact(&mut data)?;
        };

        Ok(Self { method, url, headers, data, version, stream, status:200 })
    }
    pub fn url(&self) -> &str { &self.url }
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
    pub fn set_status(&mut self, status: u16) { self.status = status;  }
    pub fn status_msg(&self) -> &'static str {
        match self.status {
            200 => "OK",
            404 => "NOT FOUND",
            501 => "NOT IMPLEMENTED",
            403 => "FORBIDDEN",
            _ => "?"
        }
    }
    pub fn content_length(&self) -> usize {
        match self.headers.get("Content-Length") {
            Some(l) => l.parse().unwrap_or(0),
            None => 0,
        }
    }
    pub fn header(&self, key: &str) -> Option<&String> {
        self.headers.get(key)
    }
    pub fn data(&self) -> &[u8] { &self.data }
    pub fn respond(&mut self, buf: &[u8]) -> Result<()> {
        let response_line = format!("HTTP/{} {} {}\r\n", self.version, self.status, self.status_msg());
        self.stream.write_all(response_line.as_bytes())?;
        if buf.len() == 0 {
            return Ok(());
        }
        let headers = format!("Content-Length: {}\r\n\r\n", buf.len());
        self.stream.write_all(headers.as_bytes())?;
        self.stream.write_all(&buf)?;
        Ok(())
    }
    pub fn ok(&mut self) -> Result<()> {
        self.status = 200;
        self.respond(&[])
    }
    pub fn forbidden(&mut self) -> Result<()> {
        self.send_error_page(403)
    }
    pub fn send_error_page(&mut self, error: u16) -> Result<()> {
        self.status = error;
        let buf = self.error_page();
        self.respond(&buf)
    }
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
