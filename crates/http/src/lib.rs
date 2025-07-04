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
//!             .build();
//! let tcp = TcpStream::connect("127.0.0.1:80").unwrap();
//! req.send_to(tcp).unwrap();
//! ```

pub mod encoding;
mod error;
pub use error::HttpError;
mod method;
pub use method::HttpMethod;
pub mod request;
pub use request::HttpRequest;
pub mod response;
pub use response::HttpResponse;
mod status;
pub use status::StatusCode;
mod stream;
pub use stream::HttpStream;

#[doc(hidden)]
pub mod prelude {
    pub use crate::{HttpError, HttpMethod, HttpRequest, HttpResponse, HttpStream, StatusCode};
}

pub type Result<T> = std::result::Result<T, HttpError>;
