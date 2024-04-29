use std::{env,thread,time::Duration};
use http_server::{
    request::{RequestMethod,handler},
    server::HttpServer,
    config::Config
};

fn main() {
    let conf = Config::parse(env::args());
    let mut server = HttpServer::new(conf.port(), conf.n_threads());

    server.add_default(RequestMethod::GET, handler::cat_handler);
    server.add_default(RequestMethod::POST, handler::post_handler);
    server.get("/sleep", |req| {
        thread::sleep(Duration::from_secs(5));
        req.ok()
    });
    server.run();
}

