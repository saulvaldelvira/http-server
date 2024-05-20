//! Base 64
//!
//! This crate contains functions to encode and decode Base 64

use std::borrow::Cow;

mod decode;
pub use decode::decode;
mod encode;
pub use encode::encode;

type Result<T> = std::result::Result<T,Cow<'static,str>>;

