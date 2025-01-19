mod stream;
pub use stream::HttpStream;

mod status;
pub use status::StatusCode;

mod method;
pub use method::HttpMethod;

pub mod encoding;

pub mod request;
pub use request::HttpRequest;

pub mod response;
pub use response::HttpResponse;

mod error;
pub use error::ServerError;

pub type Result<T> = std::result::Result<T, ServerError>;
