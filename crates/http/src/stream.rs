use std::{
    io::{self, Read, Write},
    net::TcpStream,
    time::Duration,
};

#[cfg(feature = "tls")]
use rustls::{ConnectionCommon, SideData};

pub trait HttpStream: Read + Write {
    fn set_blocking(&mut self) -> io::Result<()> {
        Ok(())
    }

    fn set_non_blocking(&mut self, timeout: Duration) -> io::Result<()> {
        let _ = timeout;
        Ok(())
    }
}

pub trait IntoHttpStream {
    type Stream: HttpStream + 'static;
    fn into_http_stream(self) -> Self::Stream;
}

#[derive(Debug)]
pub struct StringStream {
    input: Vec<u8>,
    offset: usize,
    output: Vec<u8>,
}

impl Read for StringStream {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let n = self.peek(buf)?;
        self.offset += n;
        Ok(n)
    }

    fn read_to_end(&mut self, buf: &mut Vec<u8>) -> io::Result<usize> {
        const CHUNK_SIZE: usize = 1024;
        let mut chunk: [u8; CHUNK_SIZE] = [0; CHUNK_SIZE];
        let n = self.offset;
        while self.read(&mut chunk)? > 0 {
            buf.write_all(&chunk)?;
        }
        Ok(self.offset - n)
    }
}

impl Write for StringStream {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.output.extend_from_slice(buf);
        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

impl StringStream {
    pub fn peek(&self, buf: &mut [u8]) -> io::Result<usize> {
        let min = usize::min(buf.len(), self.input.len() - self.offset);
        buf[..min].copy_from_slice(&self.input[self.offset..self.offset + min]);
        Ok(min)
    }
}

impl HttpStream for StringStream {}

impl IntoHttpStream for String {
    type Stream = StringStream;

    fn into_http_stream(self) -> Self::Stream {
        let src_vec = self.into_bytes();
        StringStream {
            input: src_vec,
            offset: 0,
            output: Vec::new(),
        }
    }
}

impl IntoHttpStream for &str {
    type Stream = StringStream;

    fn into_http_stream(self) -> Self::Stream {
        self.to_string().into_http_stream()
    }
}

impl<S: HttpStream + 'static> IntoHttpStream for S {
    type Stream = S;

    fn into_http_stream(self) -> Self::Stream {
        self
    }
}

impl HttpStream for TcpStream {
    fn set_non_blocking(&mut self, timeout: Duration) -> io::Result<()> {
        self.set_read_timeout(Some(timeout))
    }

    fn set_blocking(&mut self) -> io::Result<()> {
        self.set_read_timeout(None)
    }
}

pub struct DummyStream;

impl Read for DummyStream {
    fn read(&mut self, _buf: &mut [u8]) -> io::Result<usize> {
        Ok(0)
    }
}

impl Write for DummyStream {
    fn write(&mut self, _buf: &[u8]) -> io::Result<usize> {
        Ok(0)
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

impl HttpStream for DummyStream {}

pub fn dummy() -> Box<dyn HttpStream> {
    Box::new(DummyStream)
}

#[cfg(feature = "tls")]
impl<C, SD, S: HttpStream> HttpStream for rustls::StreamOwned<C, S>
where
    SD: SideData,
    C: core::ops::DerefMut<Target = ConnectionCommon<SD>>,
{
    fn set_blocking(&mut self) -> io::Result<()> {
        self.sock.set_blocking()
    }

    fn set_non_blocking(&mut self, timeout: Duration) -> io::Result<()> {
        self.sock.set_non_blocking(timeout)
    }
}
