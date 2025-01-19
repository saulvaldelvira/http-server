use std::{
    cmp::min,
    io::{self, Read, Write},
    net::TcpStream,
    time::Duration,
};

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
enum HttpStreamInner {
    Tcp(TcpStream),
    String(StringStream, Vec<u8>),
    Dummy,
}

/// Holds a "stream" for a [`request`]
/// This is an object where the http request can read and write to.
///
/// It can be build with a [`TcpStream`] or a [String]
///
/// It can also serve as a dummy object to build `HttpRequests`
/// that are not parsed from any source
///
/// ```
/// use http_srv::http::*;
/// use http_srv::request::*;
/// use std::net::TcpStream;
///
/// fn from_str() -> HttpRequest {
///     let stream = HttpStream::from("GET / HTTP/1.0");
///     HttpRequest::parse(stream).unwrap()
/// }
/// fn from_tcp(tcp: TcpStream) -> HttpRequest {
///     let stream = HttpStream::from(tcp);
///     HttpRequest::parse(stream).unwrap()
/// }
/// fn dummy() -> HttpRequest {
///     /* This HttpRequest holds a dummy HttpStream.
///        All read/write operations are no-ops */
///     HttpRequest::builder()
///         .method(HttpMethod::GET)
///         .url("/")
///         .version(1.0)
///         .build().unwrap()
/// }
/// ```
///
/// [`request`]: crate::HttpRequest
#[derive(Debug)]
pub struct HttpStream {
    inner: HttpStreamInner,
}

impl HttpStream {
    pub fn set_read_timeout(&self, d: Option<Duration>) -> io::Result<()> {
        match &self.inner {
            HttpStreamInner::Tcp(tcp_stream) => tcp_stream.set_read_timeout(d),
            HttpStreamInner::Dummy | HttpStreamInner::String(..) => Ok(()),
        }
    }
    pub fn peek(&self, buf: &mut [u8]) -> std::io::Result<usize> {
        match &self.inner {
            HttpStreamInner::Tcp(tcp_stream) => tcp_stream.peek(buf),
            HttpStreamInner::Dummy => Ok(0),
            HttpStreamInner::String(read, _) => read.peek(buf),
        }
    }
}

impl Read for HttpStream {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        match &mut self.inner {
            HttpStreamInner::Tcp(tcp_stream) => tcp_stream.read(buf),
            HttpStreamInner::Dummy => Ok(0),
            HttpStreamInner::String(buf_reader, _) => buf_reader.read(buf),
        }
    }
}

impl Write for HttpStream {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        match &mut self.inner {
            HttpStreamInner::Tcp(tcp_stream) => tcp_stream.write(buf),
            HttpStreamInner::Dummy => Ok(0),
            HttpStreamInner::String(_, w) => w.write(buf),
        }
    }

    fn flush(&mut self) -> std::io::Result<()> {
        match &mut self.inner {
            HttpStreamInner::Tcp(tcp_stream) => tcp_stream.flush(),
            HttpStreamInner::Dummy | HttpStreamInner::String(..) => Ok(()),
        }
    }
}

impl From<TcpStream> for HttpStream {
    fn from(value: TcpStream) -> Self {
        Self {
            inner: HttpStreamInner::Tcp(value),
        }
    }
}

impl From<String> for HttpStream {
    fn from(value: String) -> Self {
        let src_vec = value.into_bytes();
        let stream = StringStream(src_vec, 0);
        Self {
            inner: HttpStreamInner::String(stream, Vec::new()),
        }
    }
}

impl<'a> From<&'a str> for HttpStream {
    fn from(value: &'a str) -> Self {
        Self::from(value.to_string())
    }
}

impl HttpStream {
    #[must_use]
    pub fn dummy() -> Self {
        Self {
            inner: HttpStreamInner::Dummy,
        }
    }
}
