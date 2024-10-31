pub mod config;
pub use config::ServerConfig;
pub mod error;
pub use error::ServerError;

use crate::{log_error, request::{handler::Handler, HttpRequest}, HttpStream, Result};
use std::{sync::Arc, time::{Duration, Instant}};
use std::net::{TcpListener, TcpStream};
use pool::ThreadPool;

/// HTTP Server
///
/// Represents an HTTP Server, bound to a TCP Port
///
/// # Example
/// ```rust,no_run
/// use http_srv::server::HttpServer;
/// use http_srv::server::ServerConfig;
/// use http_srv::request::handler::Handler;
///
/// let config = ServerConfig::default();
/// let mut server = HttpServer::new(config);
/// let mut handler = Handler::new();
/// handler.get("/", |req| {
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

fn peek_stream(stream: &HttpStream, duration: Duration) -> bool {
    stream.set_read_timeout(Some(duration)).unwrap();
    let mut buf: [u8;1] = [0];
    let result = match stream.peek(&mut buf) {
        Ok(n) => n > 0,
        Err(_) => false
    };
    stream.set_read_timeout(None).unwrap();
    result
}

fn handle_connection(stream: TcpStream, handlers: Arc<Handler>, keep_alive_timeout: Duration, keep_alive_requests: u16) -> Result<()> {
    let mut req = HttpRequest::parse(stream)?;
    handlers.handle(&mut req)?;
    let connection = req.header("Connection");
    let keep_alive = keep_alive_timeout.as_millis() > 0;
    if connection.is_some() && connection.unwrap() == "keep-alive" && keep_alive {
        let start = Instant::now();
        let mut n = 1;
        while start.elapsed() < keep_alive_timeout && n < keep_alive_requests {
            let offset = keep_alive_timeout - start.elapsed();
            if !peek_stream(req.stream(), offset) { break; }

            req = req.keep_alive()?;
            handlers.handle(&mut req)?;
            n += 1;

            let connection = req.header("Connection");
            if connection.is_some() && connection.unwrap() == "close" {
               break;
            }
        }
    }
    Ok(())
}

impl HttpServer {
    /// Create a new HTTP Server
    ///
    /// # Panics
    /// - If the server fails to bind to the TCP port
    /// - If the thread pool fails to initialize
    ///
    pub fn new(config: ServerConfig) -> Self {
        let address = format!("::0:{}", config.port);
        let listener = TcpListener::bind(address)
                        .unwrap_or_else(|err| {
                            panic!("Could not bind to port {}: {}", config.port, err);
                        });
        let pool = ThreadPool::new(config.pool_conf)
                              .expect("Error initializing thread pool");
        let handler = Some(Handler::new());
        Self {listener,pool,handler,config}
    }
    /// Starts the server
    pub fn run(&mut self) {
        let handler = Arc::new(self.handler.take().unwrap());
        println!("Sever listening on port {}", self.config.port);
        for stream in self.listener.incoming() {
            match stream {
                Ok(stream) => {
                    let handler = Arc::clone(&handler);
                    let timeout = self.config.keep_alive_timeout;
                    let req = self.config.keep_alive_requests;
                    self.pool.execute(move || {
                        handle_connection(stream, handler, timeout, req).unwrap_or_else(|err| {
                            log_error!("{err}");
                        })
                    });
                },
                Err(_) => continue,
            }
        }
        println!("Shutting down.");
    }
    /// Set a [Handler] for all the [requests](HttpRequest) on this [server](HttpServer)
    pub fn set_handler(&mut self, handler: Handler) { self.handler = Some(handler); }
}

impl Default for HttpServer {
    /// Default Server
    ///
    /// - Configuration: [ServerConfig::default]
    /// - Handler: [Handler::default]
    fn default() -> Self {
        let conf = ServerConfig::default();
        let mut srv = Self::new(conf);
        let handler = Handler::default();
        srv.set_handler(handler);
        srv
    }
}
