use crate::{pool::ThreadPool, request::{handler::Handler, HttpRequest}};
use std::sync::{Arc,RwLock};
use std::net::{TcpListener, TcpStream};

/// HTTP Server
///
/// Represents an HTTP Server, bound to a TCP Port
///
/// # Example
/// ```rust,no_run
/// use http_server::server::HttpServer;
///
/// let mut server = HttpServer::new(80, 32);
/// server.get("/", |req| {
///     req.ok()
/// });
/// server.run();
/// ```
pub struct HttpServer {
    listener: TcpListener,
    pool: ThreadPool,
    handler: Option<Handler>,
    port: u16,
}

fn handle_connection(stream: TcpStream, handlers: Arc<RwLock<Handler>>) {
    let mut req = HttpRequest::parse(stream).expect("Error parsing");
    handlers.read().unwrap().handle(&mut req).unwrap_or_else(|err| {
        eprintln!("{err}");
    });
}

impl HttpServer {
    /// Create a new HTTP Server
    ///
    /// - port: TCP port to listen in
    /// - n_threads: Number of threads for the pool
    ///
    /// # Panics
    /// - If the server fails to bind to the TCP port
    /// - If the thread pool fails to initialize
    ///
    pub fn new(port: u16, n_threads: usize) -> Self {
        let address = format!("localhost:{}", port);
        let listener = TcpListener::bind(address)
                        .unwrap_or_else(|err| {
                            panic!("Could not bind to port {}: {}", port, err);
                        });
        let pool = ThreadPool::new(n_threads)
                              .expect("Error initializing thread pool");
        let handler = Some(Handler::new());
        Self {listener,pool,handler,port}
    }
    /// Starts the server
    pub fn run(&mut self) {
        let handler = Arc::new(RwLock::new(self.handler.take().unwrap()));
        println!("Sever listening on port {}", self.port);
        for stream in self.listener.incoming() {
            match stream {
                Ok(stream) => {
                    let handler = Arc::clone(&handler);
                    self.pool.execute(|| handle_connection(stream, handler));
                },
                Err(_) => continue, /* eprintln!("{err}"), */
            }
        }
        println!("Shutting down.");
    }
    /// Set a [Handler] for all the [requests](HttpRequest) on this [server](HttpServer)
    pub fn set_handler(&mut self, handler: Handler) { self.handler = Some(handler); }
}
