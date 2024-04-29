use std::env;

use http_server::server::HttpServer;
use http_server::config::Config;

fn main() {
    let conf = Config::parse(env::args());
    let server = HttpServer::new(&conf);
    server.run();
}

