use std::{
    env, net::{TcpListener, TcpStream}
};

use http_server::{http::Request, pool::ThreadPool};

use crate::config::Config;
pub mod config;

fn main() {
    let conf = Config::parse(env::args());
    let address = format!("127.0.0.1:{}", conf.port());
    let listener = TcpListener::bind(address)
                                .unwrap_or_else(|err| {
                                    panic!("Could not bind to port {}: {}", conf.port(), err);
                                });
    println!("Sever listening on port {}", conf.port());
    let pool = ThreadPool::new(4).unwrap();
    for stream in listener.incoming() {
        let stream = stream.unwrap();
        pool.execute(|| handle_connection(stream));
    }
    println!("Shutting down.");
}

fn handle_connection(stream: TcpStream) {
    let mut req = Request::parse(stream).expect("Error parsing");
    req.process().unwrap_or_else(|err| println!("Error parsing request: {}", err.get_message()));
    println!("{} {} {} {}", req.method(), req.url(), req.status(), req.status_msg());
}
