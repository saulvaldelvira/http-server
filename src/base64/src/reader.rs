use std::io::{Error, ErrorKind, Read};

use crate::encode::encode_chunk;

/// This is a Base 64 Encoder Reader
///
/// It takes a [reader](Read) and converts it's
/// output to Base 64.
pub struct Base64Encoder<T: Read> {
    reader: T,
    finished: bool
}

impl<T: Read> Base64Encoder<T> {
    /// Creates a [Base64Encoder] with a given [reader](Read)
    pub fn new(reader: T) -> Self {
        Self { reader, finished: false }
    }
}

impl<T: Read> Read for Base64Encoder<T> {
    /// Read the next chunk of data into buf.
    ///
    /// When the reader has finished, returns 0.
    ///
    /// If the length of buf is less than 4, this
    /// function returns an Error.
    ///
    /// **TODO**: Fix this
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        if buf.len() < 4 {
            return Err(Error::new(ErrorKind::InvalidInput, "Buffer too small"))
        }
        let mut group = [0_u8; 3];
        let mut count = 0;

        while count < buf.len() / 4 && !self.finished {
            let n = self.reader.read(&mut group)?;
            if n == 0 {
                self.finished = true;
                break;
            }
            let chunk = encode_chunk(&group[0..n]);
            for i in 0..4 {
                buf[count + i] = chunk[i] as u8;
                if chunk[i] == '=' {
                    self.finished = true;
                }
            }
            count += 4;
        }
        Ok(count)
    }
}
