use std::{
    borrow::Cow,
    fmt::{Debug, Display},
    io,
    num::ParseIntError,
    path::StripPrefixError,
    str::Utf8Error,
    string::FromUtf8Error,
};

/// Server Error
pub struct ServerError(Cow<'static, str>);

impl ServerError {
    /// Creates a [`ServerError`] from a &'static [str]
    #[inline]
    pub fn new(msg: impl Into<Cow<'static, str>>) -> Self {
        Self(msg.into())
    }
    /// Turns the [`ServerError`] into a [Result]<T`ServerError`or]>
    #[inline]
    pub fn err<T>(self) -> Result<T, Self> {
        Err(self)
    }
    /// Gets the message inside the [`ServerError`]
    #[inline]
    #[must_use]
    pub fn get_message(&self) -> &str {
        &self.0
    }
}

#[macro_export]
macro_rules! err {
    ($($e:tt)*) => {
        $crate::ServerError::new(format!($($e)*)).err()
    };
    ($lit:literal) => {
        $crate::ServerError::new($lit).err()
    };
    ($e:expr) => {
        $crate::ServerError::new($e).err()
    };
}

impl From<io::Error> for ServerError {
    #[inline]
    fn from(value: io::Error) -> Self {
        Self::new(value.to_string())
    }
}
impl From<FromUtf8Error> for ServerError {
    #[inline]
    fn from(value: FromUtf8Error) -> Self {
        Self::new(value.to_string())
    }
}
impl From<Utf8Error> for ServerError {
    #[inline]
    fn from(value: Utf8Error) -> Self {
        Self::new(value.to_string())
    }
}
impl From<std::path::StripPrefixError> for ServerError {
    #[inline]
    fn from(value: StripPrefixError) -> Self {
        Self::new(value.to_string())
    }
}
impl From<ParseIntError> for ServerError {
    #[inline]
    fn from(value: ParseIntError) -> Self {
        Self::new(value.to_string())
    }
}
impl From<Cow<'static, str>> for ServerError {
    #[inline]
    fn from(value: Cow<'static, str>) -> Self {
        Self(value)
    }
}
impl Debug for ServerError {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.get_message())
    }
}

impl Display for ServerError {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.get_message())
    }
}

impl From<&'static str> for ServerError {
    fn from(value: &'static str) -> Self {
        Self(value.into())
    }
}

impl From<String> for ServerError {
    fn from(value: String) -> Self {
        Self(value.into())
    }
}

impl std::error::Error for ServerError {}
