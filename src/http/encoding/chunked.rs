use std::io::{Read, Write, Result};

const CHUNK_SIZE: usize = 1024;
pub struct Chunked<R: Read> {
   reader: R,
   chunk: Vec<u8>,
   offset: usize,
   finish: bool,
}

impl<R: Read> Chunked<R> {
    pub fn new(reader: R) -> Self {
        Self {
            reader,
            chunk: Vec::with_capacity(CHUNK_SIZE + 8),
            offset: 0,
            finish: false
        }
    }
    fn next_chunk(&mut self) -> Result<bool> {
        if self.finish { return Ok(false); }
        self.chunk.clear();
        self.offset = 0;

        let mut tmpbuf: [u8; CHUNK_SIZE] = [0; CHUNK_SIZE];
        let n = self.reader.read(&mut tmpbuf)?;
        if n == 0 {
            self.finish = true;
        }
        self.chunk.write_all(format!("{n:X}\r\n").as_bytes())?;
        self.chunk.write_all(&tmpbuf[0..n])?;
        self.chunk.write_all(b"\r\n")?;
        Ok(true)
    }
}

impl<R: Read> Read for Chunked<R> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        if self.offset >= self.chunk.len() && !self.next_chunk()? { return Ok(0); }
        let mut n = self.chunk.len() - self.offset;
        if n > buf.len() {
            n = buf.len();
        }
        buf.as_mut().write_all(&self.chunk[self.offset..self.offset+n])?;
        self.offset += n;
        Ok(n)
    }
}
