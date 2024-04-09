use std::fmt::Debug;

pub type Result<T> = std::result::Result<T,ThreadPoolError>;

pub struct ThreadPoolError {
    msg: String
}

impl Debug for ThreadPoolError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ThreadPoolError: {}", self.msg)
    }
}

impl ThreadPoolError {
    pub fn new(msg: &str) -> Self {
        let msg = msg.to_owned();
        Self { msg }
    }
    pub fn get_error(&self) -> &str { &self.msg }
}

