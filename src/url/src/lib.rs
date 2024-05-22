//! Url Utils Crate
//!
//! This crate contains functions for url
//! percent encoding and decoding,

type Result<T> = std::result::Result<T,Cow<'static,str>>;

mod decode;
use std::borrow::Cow;

pub use decode::decode;

mod encode;
pub use encode::encode;
