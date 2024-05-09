use std::{env, thread, time::Duration};
use http_server::{
    ServerConfig, request::{handler::{self, Handler}, RequestMethod}, HttpServer
};

fn main() {
    let config = ServerConfig::parse(env::args().skip(1));
    let mut server = HttpServer::new(config);

    let mut handler = Handler::new();
    handler.pre_interceptor(handler::suffix_html);

    handler.add_default(RequestMethod::GET, handler::cat_handler);
    handler.add_default(RequestMethod::POST, handler::post_handler);
    handler.add_default(RequestMethod::DELETE, handler::delete_handler);
    handler.add_default(RequestMethod::HEAD, handler::head_handler);

    handler.get("/", handler::index_handler);
    handler.add(RequestMethod::HEAD, "/", handler::index_handler);
    handler.get("/sleep", |req| {
        thread::sleep(Duration::from_secs(5));
        req.ok()
    });
    handler.get("/params", |req| {
        let mut s = "".to_string();
        for (k,v) in req.params() {
            s.push_str(k);
            s.push_str(" = ");
            s.push_str(v);
        };
        req.respond_buf(s.as_bytes())
    });

    handler.post_interceptor(handler::log_request);

    server.set_handler(handler);
    server.run();
}

