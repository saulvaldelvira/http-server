use std::env;

use http_server::server::HttpServer;
pub mod config;
use crate::config::Config;

fn main() {
    let conf = Config::parse(env::args());
    let server = HttpServer::new(conf.port());
    server.run();
}

