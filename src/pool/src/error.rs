use std::{fmt::{Debug, Display}, io};

/// Pool Error
pub enum PoolError {
    /// [PoolError] containing an &'static [str]
    Str(&'static str),
    /// [PoolError] containing an owned [String]
    String(String),
}

impl PoolError {
    /// Creates a [PoolError] from a &'static [str]
    #[inline]
    pub fn from_str(msg: &'static str) -> Self {
        Self::Str(msg)
    }
    /// Creates a [PoolError] from a [String]
    #[inline]
    pub fn from_string(msg: String) -> Self {
        Self::String(msg)
    }
    /// Turns the [PoolError] into a [Result]<T,[PoolError]>
    #[inline]
    pub fn err<T>(self) -> Result<T,Self> {
        Err(self)
    }
    /// Gets the message inside the [PoolError]
    #[inline]
    pub fn get_message(&self) -> &str {
        match &self {
            Self::Str(msg) => msg,
            Self::String(msg) => &msg,
        }
    }
}

impl From<io::Error> for PoolError {
    #[inline]
    fn from(value: io::Error) -> Self {
        Self::from_string(value.to_string())
    }
}

impl Debug for PoolError {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
       write!(f, "{}", self.get_message())
    }
}

impl Display for PoolError {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
       write!(f, "{}", self.get_message())
    }
}

impl std::error::Error for PoolError { }
