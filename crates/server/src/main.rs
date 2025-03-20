use std::{env, process, thread, time::Duration};

use encoding::StreamReader;
use http_srv::prelude::*;

pub fn main() {
    let args: Vec<_> = env::args().skip(1).collect();
    let config = ServerConfig::parse(&args).unwrap_or_else(|err| {
        eprintln!("{err}");
        process::exit(1);
    });

    let mut handler = Handler::default();
    handler.get("/sleep", |req: &mut HttpRequest| {
        thread::sleep(Duration::from_secs(5));
        req.ok()
    });

    handler.get("/params", |req: &mut HttpRequest| {
        let mut s = "".to_string();
        for (k, v) in req.params() {
            s.push_str(k);
            s.push_str(" = ");
            s.push_str(v);
            s.push('\n');
        }
        req.respond_str(&s)
    });

    handler.get("/inf", |req: &mut HttpRequest| {
        let mut i = 0;
        let chars = b"hello, world!\n";
        let inf = || {
            if i >= chars.len() {
                i = 0;
            }
            i += 1;
            Some(chars[i - 1])
        };
        let mut reader = StreamReader::new(inf);
        req.respond_reader(&mut reader)
    });

    handler.get("/hello", |req: &mut HttpRequest| {
        let name = req.param("name").unwrap_or("friend");
        let msg = format!("Hello {name}!");
        req.respond_str(&msg)
    });

    handler.get("/redirect", handler::redirect("/hello"));

    #[cfg(feature = "regex")]
    handler.get(
        handler::UrlMatcher::regex(".*\\.php$").unwrap(),
        |req: &mut HttpRequest| req.set_status(500).respond_str("PHP is not supported yet"),
    );

    if let Some(file) = &config.log_file {
        handler.post_interceptor(handler::log_file(file).unwrap_or_else(|err| {
            eprintln!("{err}");
            process::exit(1);
        }));
    }

    /* For debugging */
    /* handler.post_interceptor(|req: &mut HttpRequest| { */
    /*     println!("{:?}", req.headers()); */
    /* }); */

    let auth = AuthConfig::of_list(&[("test", "test"), ("abc", "abc")]);
    handler.get(
        "/priv",
        auth.apply(|req: &mut HttpRequest| req.respond_str("Secret message")),
    );

    let mut server = HttpServer::new(config).unwrap_or_else(|err| {
        eprintln!("ERROR: {err}");
        std::process::exit(1)
    });
    server.set_handler(handler);
    server.run();
}
