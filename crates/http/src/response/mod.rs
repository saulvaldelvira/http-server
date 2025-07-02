use core::fmt;
use std::{
    collections::HashMap,
    io::{self, BufRead, BufReader, Read, Write},
};

use parse::parse_response;

use crate::{HttpStream, Result, response::builder::HttpResponseBuilder, stream::IntoHttpStream};

pub mod builder;
mod parse;

/// An Http response
pub struct HttpResponse {
    headers: HashMap<Box<str>, Box<str>>,
    stream: BufReader<Box<dyn HttpStream>>,
    status: u16,
    body: Option<Box<[u8]>>,
    version: f32,
}

impl fmt::Debug for HttpResponse {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("HttpResponse")
            .field("headers", &self.headers)
            .field("status", &self.status)
            .field("body", &self.body)
            .field("version", &self.version)
            .finish()
    }
}

impl HttpResponse {
    pub fn builder() -> HttpResponseBuilder {
        HttpResponseBuilder::new()
    }

    pub fn parse<S: IntoHttpStream>(stream: S) -> crate::Result<Self> {
        let stream: Box<dyn HttpStream> = Box::new(stream.into_http_stream());
        parse_response(BufReader::new(stream))
    }
    #[inline]
    #[must_use]
    pub fn status(&self) -> u16 {
        self.status
    }

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
        self.headers.get(key).map(|s| &**s)
    }

    #[inline]
    #[must_use]
    pub fn version(&self) -> f32 {
        self.version
    }

    #[inline]
    #[must_use]
    pub fn headers(&self) -> &HashMap<Box<str>, Box<str>> {
        &self.headers
    }

    /// Reads the body from the stream into the buffer.
    ///
    /// This method is primarly used by [`body`](Self::body), and
    /// for unit tests, where we need to force-load the body into
    /// the [stream]'s buffer.
    ///
    /// [stream]: HttpStream
    pub(crate) fn read_body_into_buffer(&mut self) -> std::io::Result<()> {
        let len = self.content_length();
        let mut buf = Vec::with_capacity(len);
        self.stream.read_to_end(&mut buf)?;
        self.body = Some(buf.into_boxed_slice());
        Ok(())
    }

    /// Reads the body from the [`stream`] into the response's buffer.
    ///
    /// # NOTE
    /// This loads the whole body of the response into memory,
    /// and it'll stick with the response for it's lifetime.
    /// It's not very efficient memory-wise for responses with big bodies.
    ///
    /// For a nicer way to process a response's body, see the
    /// [read_body](Self::read_body) function.
    ///
    /// # Errors
    /// If some IO error happens when reading the body from the [`stream`]
    ///
    /// # Returns
    /// And option of &[u8]. A None variant means the response doesn't have a body.
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
    /// since this function doesn't allocate memory, nor mutates the response.
    ///
    /// # Errors
    /// If some IO error happens in the process of checking the
    /// [`stream`]'s availability
    ///
    /// [`stream`]: HttpStream
    pub fn has_body(&mut self) -> Result<bool> {
        Ok(self.body.is_some() || !self.stream.fill_buf()?.is_empty())
    }

    /// Reads the response's body into [writer](Write)
    ///
    /// # Errors
    /// If, while reading or writing, some io Error is found
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

    pub fn write_to(&mut self, out: &mut dyn io::Write) -> io::Result<usize> {
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
}

impl PartialEq for HttpResponse {
    fn eq(&self, other: &Self) -> bool {
        self.headers == other.headers
            && self.status == other.status
            && self.body == other.body
            && self.version == other.version
    }
}

#[cfg(test)]
mod test;
