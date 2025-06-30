use std::io::{Read, Result, Write};

/// A reader for [HTTP Chunked transfer encoding]
/// 
/// [HTTP Chunked transfer encoding]: <https://en.wikipedia.org/wiki/Chunked_transfer_encoding>
pub struct Chunked<R: Read, const CHUNK_SIZE: usize = 1024> {
    reader: R,
    chunk: Vec<u8>,
    offset: usize,
}

impl<R: Read> Chunked<R> {
    /// Creates a `Chunked` struct with the default size.
    pub fn with_default_size(reader: R) -> Chunked<R> {
        Self::new(reader)
    }
}

impl<R: Read, const CHUNK_SIZE: usize> Chunked<R, CHUNK_SIZE> {

    /// The size of the chunks
    pub const CHUNK_SIZE: usize = CHUNK_SIZE;

    pub fn new(reader: R) -> Self {
        Chunked {
            reader,
            chunk: Vec::with_capacity(CHUNK_SIZE + 8),
            offset: 0,
        }
    }
    fn next_chunk(&mut self) -> Result<bool> {
        self.chunk.clear();
        self.offset = 0;

        let mut tmpbuf: [u8; CHUNK_SIZE] = [0; CHUNK_SIZE];
        let n = self.reader.read(&mut tmpbuf)?;
        if n == 0 {
            return Ok(false)
        }
        self.chunk.write_all(format!("{n:X}\r\n").as_bytes())?;
        self.chunk.write_all(&tmpbuf[0..n])?;
        self.chunk.write_all(b"\r\n")?;
        Ok(true)
    }

    /// Returns the current chunk
    /// 
    /// # NOTE
    /// This method returns the whole chunk, even the parts alredy 
    /// read. If you want to know the remaining portion of the chunk
    /// that hasn't been polled, see [offset](Self::offset)
    pub fn current_chunk(&self) -> &[u8] { &self.chunk }

    /// Returns the current offset. This is: The offset to the 
    /// part of the current chunk that hasn't been read yet
    pub fn offset(&self) -> usize { self.offset }
}

impl<R: Read, const CHUNK_SIZE: usize> Read for Chunked<R, CHUNK_SIZE> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        if self.offset >= self.chunk.len() && !self.next_chunk()? {
            return Ok(0);
        }
        let mut n = self.chunk.len() - self.offset;
        if n > buf.len() {
            n = buf.len();
        }
        buf.as_mut()
            .write_all(&self.chunk[self.offset..self.offset + n])?;
        self.offset += n;
        Ok(n)
    }
}

impl<R: Read + Default> Default for Chunked<R> {
    fn default() -> Self {
        Self::new(R::default())
    }
}

#[cfg(test)]
mod test {
    pub use super::*;

    const SIZE: usize = 1024;

    fn test_chunks(input: &str) {
        let mut chunked = Chunked::<_, SIZE>::new(input.as_bytes());
        let mut out = Vec::new();

        chunked.read_to_end(&mut out).unwrap();

        let mut expected = Vec::new();
        for chunk in input.as_bytes().chunks(SIZE) {
            expected.extend_from_slice(format!("{:X}\r\n", chunk.len()).as_bytes());
            expected.extend_from_slice(chunk);
            expected.extend_from_slice(b"\r\n");
        }
        assert_eq!(out, expected);
    }

    #[test]
    fn small() {
        test_chunks("abcdefg");
        test_chunks(&"0".repeat(SIZE - 50));
    }

    #[test]
    fn exact_chunks() {
        test_chunks(&"a".repeat(SIZE));
        test_chunks(&"a".repeat(SIZE * 2));
    }

    #[test]
    fn with_remaining() {
        test_chunks(&"a".repeat(SIZE + 200));
    }
}