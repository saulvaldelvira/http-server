use std::{env,thread,time::Duration};
use http_server::{
    request::{RequestMethod,handler},
    HttpServer,
    config::Config
};

fn main() {
    let conf = Config::parse(env::args());
    let mut server = HttpServer::new(conf.port(), conf.n_threads());

    server.add_default(RequestMethod::GET, handler::cat_handler);
    server.add_default(RequestMethod::POST, handler::post_handler);
    server.add_default(RequestMethod::DELETE, handler::delete_handler);
    server.get("/", |req| {
        req.respond_buf(
        b"<!DOCTYPE html>
          <html>
            <head>
                <title>HTTP Server</title>
            </head>
            <body>
                <h1>HTTP Server</h1>
                <p>Hello world :)</p>
            </body>
          </html>")
    });
    server.get("/sleep", |req| {
        thread::sleep(Duration::from_secs(5));
        req.ok()
    });
    server.run();
}

