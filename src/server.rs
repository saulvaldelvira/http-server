use std::net::{TcpListener, TcpStream};

use crate::{http::Request, pool::ThreadPool};

pub struct HttpServer {
    listener: TcpListener,
    pool: ThreadPool,
}

fn handle_connection(stream: TcpStream) {
    let mut req = Request::parse(stream).expect("Error parsing");
    req.process().unwrap_or_else(|err| println!("Error parsing request: {}", err.get_message()));
    println!("{} {} {} {}", req.method(), req.url(), req.status(), req.status_msg());
}

impl HttpServer {
    pub fn new(port: u16) -> Self {
        let address = format!("127.0.0.1:{}", port);
        let listener = TcpListener::bind(address)
            .unwrap_or_else(|err| {
                panic!("Could not bind to port {}: {}", port, err);
            });
        println!("Sever listening on port {}", port);
        let pool = ThreadPool::new(32).unwrap();
        Self {listener,pool}
    }
    pub fn run(&self) {
        for stream in self.listener.incoming() {
            let stream = stream.unwrap();
            self.pool.execute(|| handle_connection(stream));
        }
        println!("Shutting down.");
    }
}
