pub mod pool;
pub mod error;
pub mod request;
pub mod server;
pub mod config;

use error::ServerError;
pub type Result<T> = std::result::Result<T,ServerError>;
