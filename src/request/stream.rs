use std::{cmp::{max, min}, io::{self, Read, Write}, net::TcpStream, time::Duration};

#[derive(Debug)]
struct StringStream(Vec<u8>, usize);

impl Read for StringStream {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let n = self.peek(buf)?;
        self.1 += n;
        Ok(n)
    }
}

impl StringStream {
    fn peek(&self, buf: &mut [u8]) -> io::Result<usize> {
        if self.1 >= self.0.len() {
            return Err(io::Error::new(io::ErrorKind::WouldBlock, ""));
        }
        let src = &self.0[self.1..];
        let n = min(buf.len(), src.len());
        buf[..n].copy_from_slice(src);
        Ok(n)
    }
}

#[derive(Debug)]
enum RequestStreamInner {
    Tcp(TcpStream),
    String(StringStream,Vec<u8>),
    Dummy
}

/// Holds a "stream" for an [HttpRequest]
/// This is an object where the http request can read and write to.
///
/// It can be build with a [TcpStream] or a [String]
///
/// It can also serve as a dummy object to build HttpRequests
/// that are not parsed from any source
///
/// ```
/// use http_srv::request::*;
/// use std::net::TcpStream;
///
/// fn from_str() -> HttpRequest {
///     let stream = RequestStream::from("GET / HTTP/1.0");
///     HttpRequest::parse(stream).unwrap()
/// }
/// fn from_tcp(tcp: TcpStream) -> HttpRequest {
///     let stream = RequestStream::from(tcp);
///     HttpRequest::parse(stream).unwrap()
/// }
/// fn dummy() -> HttpRequest {
///     /* This HttpRequest holds a dummy RequestStream.
///        All read/write operations are no-ops */
///     HttpRequest::builder()
///         .method(RequestMethod::GET)
///         .url("/")
///         .version(1.0)
///         .build().unwrap()
/// }
/// ```
#[derive(Debug)]
pub struct RequestStream {
    inner: RequestStreamInner,
}

impl RequestStream {
    pub fn set_read_timeout(&self, d: Option<Duration>) -> io::Result<()> {
        match &self.inner {
            RequestStreamInner::Tcp(tcp_stream) => tcp_stream.set_read_timeout(d),
            RequestStreamInner::Dummy => Ok(()),
            RequestStreamInner::String(_, _) => Ok(()),
        }
    }
    pub fn peek(&self, buf: &mut [u8]) -> std::io::Result<usize> {
        match &self.inner {
            RequestStreamInner::Tcp(tcp_stream) => tcp_stream.peek(buf),
            RequestStreamInner::Dummy => Ok(0),
            RequestStreamInner::String(read, _) => read.peek(buf),
        }
    }
}

impl Read for RequestStream {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        match &mut self.inner {
            RequestStreamInner::Tcp(tcp_stream) => tcp_stream.read(buf),
            RequestStreamInner::Dummy => Ok(0),
            RequestStreamInner::String(buf_reader, _) => buf_reader.read(buf),
        }
    }
}

impl Write for RequestStream {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        match &mut self.inner {
            RequestStreamInner::Tcp(tcp_stream) => tcp_stream.write(buf),
            RequestStreamInner::Dummy => Ok(0),
            RequestStreamInner::String(_, w) => w.write(buf),
        }
    }

    fn flush(&mut self) -> std::io::Result<()> {
        match &mut self.inner {
            RequestStreamInner::Tcp(tcp_stream) => tcp_stream.flush(),
            RequestStreamInner::Dummy => Ok(()),
            RequestStreamInner::String(_, _) => todo!(),
        }
    }
}

impl From<TcpStream> for RequestStream {
    fn from(value: TcpStream) -> Self {
        Self { inner: RequestStreamInner::Tcp(value) }
    }
}

impl From<String> for RequestStream {
    fn from(value: String) -> Self {
        let src_vec = value.into_bytes();
        let stream = StringStream(src_vec, 0);
        Self { inner: RequestStreamInner::String(stream, Vec::new()) }
    }
}

impl<'a> From<&'a str> for RequestStream {
    fn from(value: &'a str) -> Self {
        Self::from(value.to_string())
    }
}

impl RequestStream {
    pub fn dummy() -> Self {
        Self { inner: RequestStreamInner::Dummy }
    }
}
