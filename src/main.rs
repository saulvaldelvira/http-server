use std::{env, thread, time::Duration};
use http_server::{
    config::Config, request::{handler::{self, Handler}, RequestMethod}, HttpServer
};

fn main() {
    let conf = Config::parse(env::args());
    let mut server = HttpServer::new(conf.port(), conf.n_threads());

    let mut handler = Handler::new();
    handler.pre_interceptor(handler::suffix_html);

    handler.add_default(RequestMethod::GET, handler::cat_handler);
    handler.add_default(RequestMethod::POST, handler::post_handler);
    handler.add_default(RequestMethod::DELETE, handler::delete_handler);

    handler.get("/", handler::index_handler);
    handler.get("/sleep", |req| {
        thread::sleep(Duration::from_secs(5));
        req.ok()
    });

    handler.post_interceptor(handler::log_request);

    server.set_handler(handler);
    server.run();
}

