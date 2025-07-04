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
//! let mut server = HttpServer::new(config).unwrap();
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
#![allow(clippy::missing_errors_doc, clippy::module_name_repetitions)]

pub use http::{self, HttpRequest, HttpResponse, request, response};
pub mod config;

pub mod handler;

#[doc(hidden)]
pub mod prelude {
    pub use http::{request::HttpRequest, response::*, *};

    pub use crate::{
        HttpServer,
        config::*,
        handler::{self, AuthConfig, Handler},
    };
}
use prelude::*;

/// Result type for the [`http_srv`](self) crate
///
/// It serves as a shortcut for an [`std::result::Result`]<T,[`HttpError`]>
pub type Result<T> = std::result::Result<T, HttpError>;

use std::{
    io::{self, BufRead, BufReader},
    net::{TcpListener, TcpStream},
    sync::Arc,
    thread,
    time::{Duration, Instant},
};

pub use config::ServerConfig;
use http::HttpStream;
use pool::ThreadPool;

mod log;
use log::prelude::*;

/// HTTP Server
///
/// Represents an HTTP Server, bound to a TCP Port
///
/// # Example
/// ```rust,no_run
/// use http_srv::HttpServer;
/// use http_srv::ServerConfig;
/// use http_srv::handler::Handler;
/// use http_srv::request::HttpRequest;
///
/// let config = ServerConfig::default();
/// let mut server = HttpServer::new(config).unwrap();
/// let mut handler = Handler::new();
/// handler.get("/", |req: &mut HttpRequest| {
///     req.ok()
/// });
/// server.set_handler(handler);
/// server.run();
/// ```
pub struct HttpServer {
    listener: TcpListener,
    pool: ThreadPool,
    handler: Option<Handler>,
    config: ServerConfig,
}

fn peek_stream(
    stream: &mut BufReader<Box<dyn HttpStream>>,
    duration: Duration,
) -> io::Result<bool> {
    stream.get_mut().set_non_blocking(duration)?;
    let result = match stream.fill_buf() {
        Ok(b) => !b.is_empty(),
        Err(err) => match err.kind() {
            io::ErrorKind::WouldBlock => false,
            _ => return Err(err),
        },
    };
    stream.get_mut().set_blocking()?;
    Ok(result)
}

fn handle_connection(
    stream: TcpStream,
    handlers: &Handler,
    keep_alive_timeout: Duration,
    keep_alive_requests: u16,

    #[cfg(feature = "tls")] tls_config: Option<&Arc<rustls::ServerConfig>>,
) -> Result<()> {
    #[cfg(feature = "tls")]
    let mut req = match tls_config {
        Some(config) => {
            let conn = rustls::ServerConnection::new(Arc::clone(config))
                .map_err(|err| format!("TLS error: {err}"))?;
            let tls_stream = rustls::StreamOwned::new(conn, stream);
            HttpRequest::parse(tls_stream)?
        }
        None => HttpRequest::parse(stream)?,
    };

    #[cfg(not(feature = "tls"))]
    let mut req = HttpRequest::parse(stream)?;

    handlers.handle(&mut req)?;

    let connection = req.header("Connection");
    let keep_alive = keep_alive_timeout.as_millis() > 0;
    if connection.is_some_and(|conn| conn == "keep-alive") && keep_alive {
        let start = Instant::now();
        let mut n = 1;
        log_info!("[{:?}] Start keep alive", thread::current().id());
        while start.elapsed() < keep_alive_timeout && n < keep_alive_requests {
            let offset = keep_alive_timeout - start.elapsed();

            match peek_stream(req.stream_mut(), offset) {
                Ok(false) => break,
                Err(err) => {
                    log_error!("Error on peek_stream: {err}");
                    break;
                }
                _ => {}
            }

            req = req.keep_alive()?;
            handlers.handle(&mut req)?;
            n += 1;

            let connection = req.header("Connection");
            if connection.is_some_and(|conn| conn == "close") {
                break;
            }
        }
        log_info!("[{:?}] End keep alive", thread::current().id());
    }
    Ok(())
}

#[allow(clippy::unwrap_used)]
#[allow(clippy::expect_used)]
#[allow(clippy::panic)]
impl HttpServer {
    /// Create a new HTTP Server
    ///
    /// # Errors
    /// - If the server fails to bind to the TCP port
    /// - If the thread pool fails to initialize
    ///
    pub fn new(config: ServerConfig) -> Result<Self> {
        let address = format!("::0:{}", config.port);
        let listener = TcpListener::bind(address)
            .map_err(|err| format!("Could not bind to port {}: {}", config.port, err))?;
        let pool =
            ThreadPool::new(config.pool_conf).map_err(|_| "Error initializing thread pool")?;
        let handler = Some(Handler::new());
        let srv = Self {
            listener,
            pool,
            handler,
            config,
        };
        Ok(srv)
    }
    /// Starts the server
    #[allow(clippy::missing_panics_doc)]
    pub fn run(self) {
        let Self {
            listener,
            pool,
            handler,
            config,
        } = self;
        let handler = handler.unwrap();
        let timeout = config.keep_alive_timeout;
        let req = config.keep_alive_requests;

        #[cfg(feature = "tls")]
        let tls_config = config.tls_config.as_ref();

        println!("Sever listening on port {}", config.port);

        pool.scope(|scope| {
            for stream in listener.incoming().flatten() {
                scope.execute(|| {
                    handle_connection(
                        stream,
                        &handler,
                        timeout,
                        req,
                        #[cfg(feature = "tls")]
                        tls_config,
                    )
                    .unwrap_or_else(|err| {
                        log_error!("{err}");
                    });
                });
            }
        });

        println!("Shutting down.");
    }
    /// Set a [Handler] for all the [requests](HttpRequest) on this [server](HttpServer)
    pub fn set_handler(&mut self, handler: Handler) {
        self.handler = Some(handler);
    }
}

impl Default for HttpServer {
    /// Default Server
    ///
    /// - Configuration: [`ServerConfig::default`]
    /// - Handler: [`Handler::default`]
    ///
    /// # Panics
    /// If the [`HttpServer`] fails to initialize
    fn default() -> Self {
        let conf = ServerConfig::default();
        #[allow(clippy::expect_used)]
        let mut srv = Self::new(conf)
            .expect("Fatal error: HttpServer failed to initialize with default config");
        let handler = Handler::default();
        srv.set_handler(handler);
        srv
    }
}
