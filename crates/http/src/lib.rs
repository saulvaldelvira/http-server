//! Http utils
//!
//! This crate contains utilities to work with the HTTP protocol
//!
//! # Example
//! ```rust,no_run
//! use http::prelude::*;
//! use std::net::TcpStream;
//!
//! let req = HttpRequest::builder()
//!             .method(HttpMethod::GET)
//!             .url("/")
//!             .build().unwrap();
//! let tcp = TcpStream::connect("127.0.0.1:80").unwrap();
//! req.send_to(HttpStream::from(tcp)).unwrap();
//! ```

pub mod encoding;
mod error;
mod method;
pub mod request;
pub mod response;
mod status;
mod stream;

pub mod prelude {
    pub use crate::{
        error::HttpError, method::HttpMethod, request::HttpRequest, response::HttpResponse,
        status::StatusCode, stream::HttpStream,
    };
}

#[doc(hidden)]
pub use prelude::*;

pub type Result<T> = std::result::Result<T, HttpError>;
