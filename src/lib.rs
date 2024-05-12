pub mod request;
pub mod server;

use server::ServerError;
pub use server::HttpServer;
pub use request::HttpRequest;
pub use request::handler::Handler;
pub use server::ServerConfig;

/// Result type for the [http_server](self) crate
///
/// It serves as a shortcut for an [std::result::Result]<T,[ServerError]>
pub type Result<T> = std::result::Result<T,ServerError>;
