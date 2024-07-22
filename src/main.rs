use std::{env, thread, time::Duration};
use http_srv::prelude::*;
use http_srv::request::encoding::StreamReader;

fn main() {
    let config = ServerConfig::parse(env::args().skip(1));

    let mut handler = Handler::default();
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
            s.push('\n');
        };
        req.respond_str(&s)
    });

    handler.get("/inf", |req| {
        let mut i = 0;
        let chars = b"hello, world!\n";
        let inf = || {
            if i >= chars.len() { i = 0; }
            i += 1;
            Some(chars[i - 1])
        };
        let mut reader = StreamReader::new(inf);
        req.respond_reader(&mut reader)
    });

    handler.get("/hello", |req| {
        let name = req.param("name").unwrap_or("friend");
        let msg = format!("Hello {name}!");
        req.respond_str(&msg)
    });

    handler.get("/redirect", handler::redirect("/hello"));

    if let Some(file) = &config.log_file {
        handler.post_interceptor(handler::log_file(file));
    }

    /* For debugging */
    /* handler.post_interceptor(|req| { */
    /*     println!("{:?}", req.headers()); */
    /* }); */

    let auth = AuthConfig::of_list(&[
                                   ("test", "test"),
                                   ("abc", "abc"),
                                    ]);
    handler.get("/priv", &auth + |req| {
        req.respond_str("Secret message")
    });

    let mut server = HttpServer::new(config);
    server.set_handler(handler);
    server.run();
}

