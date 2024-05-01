pub mod pool;
pub mod error;
pub mod request;
pub mod server;
pub mod config;

use error::ServerError;
pub use server::HttpServer;
pub use request::HttpRequest;

/// Result type for the [http_server](self) crate
///
/// It serves as a shortcut for an [std::result::Result]<T,[ServerError]>
pub type Result<T> = std::result::Result<T,ServerError>;
