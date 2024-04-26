use std::{env, ffi::OsStr, fmt::Display, fs, io::{BufRead, BufReader, Write}, net::TcpStream, path::Path};

pub use super::error::ServerError as HttpError;
pub type Result<T> = std::result::Result<T,HttpError>;

#[derive(Debug)]
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
            _ => HttpError::from_str("Error parsing request method").err()
        }
    }
}

impl Display for RequestMethod {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

pub struct Request {
    method: RequestMethod,
    url: String,
    version: f32,
    stream: TcpStream,
    status: u16,
}

impl Request {
    pub fn parse(stream: TcpStream) -> Result<Self>  {
        let req = BufReader::new(&stream);
        let mut lines = req.lines();
        let line = lines.next().unwrap().unwrap();
        let mut space = line.split_whitespace().take(3);

        let method = RequestMethod::from_str(space.next().unwrap())?;
        let url = space.next().unwrap().to_owned();
        let version: f32 = space.next().unwrap()
                               .replace("HTTP/", "")
                               .parse()
                               .or_else(|_| HttpError::from_str("Could not parse HTTP Version").err())?;
        Ok(Self { method, url, version, stream, status:200 })
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
        let cwd = cwd.to_str().ok_or_else(|| HttpError::from_str("Error getting cwd"))?;
        Ok(cwd.to_owned())
    }
    pub fn process(&mut self) -> Result<()> {
        match self.method {
            RequestMethod::GET => self.get(),
            _ => panic!("Unsupported")
        }
    }
    pub fn method(&self) -> &RequestMethod { &self.method }
    pub fn status(&self) -> u16 { self.status }
    pub fn status_msg(&self) -> &'static str {
        match self.status {
            200 => "OK",
            404 => "NOT FOUND",
            _ => "?"
        }
    }

    pub fn respond(&mut self, buf: Vec<u8>) -> Result<()> {
        let header = format!("HTTP/{} {} {}\r\n\
                          Content-Length: {}\r\n\
                          \r\n", self.version, self.status,
                                      self.status_msg(), buf.len());
        self.stream.write_all(header.as_bytes())?;
        self.stream.write_all(&buf)?;
        Ok(())
    }
    fn get(&mut self) -> Result<()> {
        let filename = self.filename()?;
        let contents = fs::read(&filename).unwrap_or_else(|_| {
            println!("Error reading {}", &filename);
            self.status = 404;
            fs::read("404.html").expect("Error reading file 404.html")
        });
        self.respond(contents)?;
        Ok(())
    }
}
