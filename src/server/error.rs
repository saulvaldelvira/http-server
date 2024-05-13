use std::{fmt::{Debug, Display}, io, num::ParseIntError, path::StripPrefixError, string::FromUtf8Error};

/// Server Error
pub enum ServerError {
    /// [ServerError] containing an &'static [str]
    Str(&'static str),
    /// [ServerError] containing an owned [String]
    String(String),
}

impl ServerError {
    /// Creates a [ServerError] from a &'static [str]
    pub fn from_str(msg: &'static str) -> Self {
        Self::Str(msg)
    }
    /// Creates a [ServerError] from a [String]
    pub fn from_string(msg: String) -> Self {
        Self::String(msg)
    }
    /// Turns the [ServerError] into a [Result]<T,[ServerError]>
    pub fn err<T>(self) -> Result<T,Self> {
        Err(self)
    }
    /// Gets the message inside the [ServerError]
    pub fn get_message(&self) -> &str {
        match &self {
            Self::Str(msg) => msg,
            Self::String(msg) => &msg,
        }
    }
}

macro_rules! err {
    ($lit:literal) => {
        ServerError::from_str($lit)
    };
    ($e:expr) => {
        ServerError::from_string($e)
    };
}

pub (crate) use err;

impl From<io::Error> for ServerError {
    fn from(value: io::Error) -> Self {
        Self::from_string(value.to_string())
    }
}
impl From<FromUtf8Error> for ServerError {
    fn from(value: FromUtf8Error) -> Self {
        Self::from_string(value.to_string())
    }
}
impl From<std::path::StripPrefixError> for ServerError {
    fn from(value: StripPrefixError) -> Self {
        Self::from_string(value.to_string())
    }
}
impl From<ParseIntError> for ServerError {
    fn from(value: ParseIntError) -> Self {
        Self::from_string(value.to_string())
    }
}
impl From<String> for ServerError {
    fn from(value: String) -> Self {
        Self::from_string(value)
    }
}
impl Debug for ServerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
       write!(f, "{}", self.get_message())
    }
}

impl Display for ServerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
       write!(f, "{}", self.get_message())
    }
}

impl std::error::Error for ServerError { }
