use rb64::Result;

#[derive(Clone, Copy)]
pub enum Operation {
    Encode, Decode
}

pub struct Config {
    operation: Operation,
    files: Vec<String>,
}

impl Config {
    pub fn parse(args: impl Iterator<Item = String>) -> Result<Self> {
        let mut conf = Self::default();
        for arg in args {
            match arg.as_str() {
                "-e" => conf.operation = Operation::Encode,
                "-d" => conf.operation = Operation::Decode,
                _ => conf.files.push(arg),
            }
        }
        Ok(conf)
    }
    pub fn operation(&self) -> Operation { self.operation }
    pub fn files(&self) -> &[String] { &self.files }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            operation: Operation::Encode,
            files: Vec::new(),
        }
    }
}
