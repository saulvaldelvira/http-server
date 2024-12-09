//! Http Server Crate
//!
//! This crate contains all the libraries necessary to run an HTTP Server
//!
//! # Example
//! ```rust,no_run
//! use http_srv::prelude::*;
//!
//! let config = ServerConfig::default();
//!
//! let mut handler = Handler::new();
//! handler.add_default(HttpMethod::GET, handler::cat_handler);
//! handler.get("/", handler::root_handler);
//! handler.get("/hello", |req: &mut HttpRequest| {
//!     let name = req.param("name").unwrap_or("friend");
//!     let msg = format!("Hello {name}!");
//!     req.respond_str(&msg)
//! });
//!
//! let mut server = HttpServer::new(config);
//! server.set_handler(handler);
//! server.run();
//! ```

#![deny(
    clippy::unwrap_used,
    clippy::panic,
    clippy::expect_used,
    unused_must_use
)]
#![warn(clippy::pedantic)]
#![allow(
    clippy::missing_errors_doc,
    clippy::module_name_repetitions,
)]

pub mod request;
pub mod server;
mod log;
pub mod http;

pub mod response;

#[doc(hidden)]
pub mod prelude {
    pub use crate:: {
        server::HttpServer,
        request::HttpRequest,
        response::HttpResponse,
        request::handler::{self,Handler,AuthConfig},
        server::ServerConfig,
        http::*,
    };
    pub (crate) use crate::server::ServerError;
    pub (crate) use crate::log::prelude::*;
}
use prelude::*;

/// Result type for the [`http_srv`](self) crate
///
/// It serves as a shortcut for an [`std::result::Result`]<T,[`ServerError`]>
pub type Result<T> = std::result::Result<T,ServerError>;

pub mod client;
