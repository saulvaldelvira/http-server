use std::{env, process, thread, time::Duration};

use http_srv::{http::encoding::StreamReader, prelude::*};

pub fn main() {
    let config = ServerConfig::parse(env::args().skip(1)).unwrap_or_else(|err| {
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

    let mut server = HttpServer::new(config);
    server.set_handler(handler);
    server.run();
}
