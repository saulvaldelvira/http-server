//! Http Server Crate
//!
//! This crate contains all the libraries necessary to run an HTTP Server
//!
//! # Example
//! ```rust,no_run
//! use http_srv::{
//!     request::{
//!         handler::{self, Handler},
//!         RequestMethod
//!     },
//!     HttpServer,
//!     ServerConfig
//! };
//!
//! fn main() {
//!     let config = ServerConfig::default();
//!
//!     let mut handler = Handler::new();
//!     handler.add_default(RequestMethod::GET, handler::cat_handler);
//!     handler.get("/", handler::index_handler);
//!     handler.get("/hello", |req| {
//!         let name = req.param("name").unwrap_or("friend");
//!         let msg = format!("Hello {name}!");
//!         req.respond_str(&msg)
//!     });
//!
//!     let mut server = HttpServer::new(config);
//!     server.set_handler(handler);
//!     server.run();
//! }
//! ```

pub mod request;
pub mod server;

use server::ServerError;
pub use server::HttpServer;
pub use request::HttpRequest;
pub use request::handler::Handler;
pub use server::ServerConfig;

/// Result type for the [http_srv](self) crate
///
/// It serves as a shortcut for an [std::result::Result]<T,[ServerError]>
pub type Result<T> = std::result::Result<T,ServerError>;
