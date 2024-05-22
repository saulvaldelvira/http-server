use std::io::Read;

use crate::encode::encode_chunk;

pub struct Base64Encoder<T: Read> {
    reader: T,
    finished: bool
}

impl<T: Read> Base64Encoder<T> {
    pub fn new(reader: T) -> Self {
        Self { reader, finished: false }
    }
}

impl<T: Read> Read for Base64Encoder<T> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
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
