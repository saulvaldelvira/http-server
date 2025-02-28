use std::io::Read;

/// Generator function
pub trait Generator<'a, T>: FnMut() -> Option<T> + 'a {}
impl<'a, T, R> Generator<'a, R> for T where T: FnMut() -> Option<R> + 'a {}

/// Reader that takes a [Generator] function
/// and reads values from it
pub struct StreamReader<'a, T> {
    generator: Box<dyn Generator<'a, T>>,
}

impl<'a, T> StreamReader<'a, T> {
    pub fn new(generator: impl Generator<'a, T>) -> Self {
        let generator = Box::new(generator);
        Self { generator }
    }
}

impl Read for StreamReader<'_, u8> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        for (i, e) in buf.iter_mut().enumerate() {
            match (self.generator)() {
                Some(b) => *e = b,
                None => return Ok(i),
            }
        }
        Ok(buf.len())
    }
}
