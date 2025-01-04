//! Url Utils Crate
//!
//! This crate contains functions for url
//! percent encoding and decoding,

#![cfg_attr(not(feature = "std"), no_std)]

#[macro_use]
extern crate alloc;

type Result<T> = core::result::Result<T,Cow<'static,str>>;

mod decode;
use alloc::borrow::Cow;

pub use decode::decode;

mod encode;
pub use encode::encode;
