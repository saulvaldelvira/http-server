use crate::{pool::ThreadPool, request::{handler::{Handler, HandlerFunc}, HttpRequest, RequestMethod}};
use std::sync::{Arc,RwLock};
use std::net::{TcpListener, TcpStream};

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
    println!("{} {} {} {}", req.method(), req.url(), req.status(), req.status_msg());
}

impl HttpServer {
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
    pub fn run(&mut self) {
        let handler = Arc::new(RwLock::new(self.handler.take().unwrap()));
        println!("Sever listening on port {}", self.port);
        for stream in self.listener.incoming() {
            let stream = stream.unwrap();
            let handler = Arc::clone(&handler);
            self.pool.execute(|| handle_connection(stream, handler));
        }
        println!("Shutting down.");
    }
    pub fn get<F: HandlerFunc>(&mut self, url: &str, f: F) {
        self.add(RequestMethod::GET, url, f);
    }
    pub fn post<F: HandlerFunc>(&mut self, url: &str, f: F) {
        self.add(RequestMethod::POST, url, f);
    }
    pub fn add<F: HandlerFunc>(&mut self, method: RequestMethod, url: &str, f: F) {
        self.handler.as_mut().unwrap().add(method, url, f);
    }
    pub fn add_default<F: HandlerFunc>(&mut self, method: RequestMethod, f: F) {
        self.handler.as_mut().unwrap().add_default(method, f);
    }
}
