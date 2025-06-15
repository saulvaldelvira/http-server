use core::fmt;
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
pub struct HttpError(Cow<'static, str>);

impl HttpError {
    /// Creates a [`HttpError`] from a &'static [str]
    #[inline]
    pub fn new(msg: impl Into<Cow<'static, str>>) -> Self {
        Self(msg.into())
    }
    /// Turns the [`HttpError`] into a [Result]<T,[`HttpError`]>
    #[inline]
    pub fn err<T>(self) -> Result<T, Self> {
        Err(self)
    }
    /// Gets the message inside the [`HttpError`]
    #[inline]
    #[must_use]
    pub fn get_message(&self) -> &str {
        &self.0
    }
}

#[macro_export]
macro_rules! err {
    ($($e:tt)*) => {
        $crate::HttpError::new(format!($($e)*)).err()
    };
    ($lit:literal) => {
        $crate::HttpError::new($lit).err()
    };
    ($e:expr) => {
        $crate::HttpError::new($e).err()
    };
}

impl From<io::Error> for HttpError {
    #[inline]
    fn from(value: io::Error) -> Self {
        Self::new(value.to_string())
    }
}
impl From<FromUtf8Error> for HttpError {
    #[inline]
    fn from(value: FromUtf8Error) -> Self {
        Self::new(value.to_string())
    }
}
impl From<Utf8Error> for HttpError {
    #[inline]
    fn from(value: Utf8Error) -> Self {
        Self::new(value.to_string())
    }
}
impl From<std::path::StripPrefixError> for HttpError {
    #[inline]
    fn from(value: StripPrefixError) -> Self {
        Self::new(value.to_string())
    }
}
impl From<ParseIntError> for HttpError {
    #[inline]
    fn from(value: ParseIntError) -> Self {
        Self::new(value.to_string())
    }
}
impl From<Cow<'static, str>> for HttpError {
    #[inline]
    fn from(value: Cow<'static, str>) -> Self {
        Self(value)
    }
}

impl Debug for HttpError {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.get_message())
    }
}

impl Display for HttpError {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.get_message())
    }
}

impl From<&'static str> for HttpError {
    fn from(value: &'static str) -> Self {
        Self(value.into())
    }
}

impl From<String> for HttpError {
    fn from(value: String) -> Self {
        Self(value.into())
    }
}

impl From<fmt::Error> for HttpError {
    fn from(value: fmt::Error) -> Self {
        Self(value.to_string().into())
    }
}

impl std::error::Error for HttpError {}
