use std::io::Read;

/// Generator function
pub trait Generator<'a, T>: FnMut() -> Option<T> + 'a {}
impl<'a, T, R> Generator<'a, R> for T where T: FnMut() -> Option<R> + 'a {}

/// Reader that takes a [Generator] function
/// and reads values from it
pub struct StreamReader<'a, T> {
    gen: Box<dyn Generator<'a, T>>,
}

impl<'a, T> StreamReader<'a, T> {
    pub fn new(gen: impl Generator<'a, T>) -> Self {
        let gen = Box::new(gen);
        Self { gen }
    }
}

impl Read for StreamReader<'_, u8> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        for (i, e) in buf.iter_mut().enumerate() {
            match (self.gen)() {
                Some(b) => *e = b,
                None => return Ok(i),
            }
        }
        Ok(buf.len())
    }
}
