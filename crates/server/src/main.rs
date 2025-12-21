use std::{env, process, thread, time::Duration};

use encoding::StreamReader;
use http_srv::prelude::*;
use libloading::{Library, Symbol};

type Result<T> = ::core::result::Result<T, libloading::Error>;

fn load_lib(handler: &mut Handler, name: &str) -> Result<Library> {
    unsafe {
        let lib = libloading::Library::new(name)?;

        let init_handler: Symbol<fn(*mut Handler)> = lib.get(b"init_handler")?;

        init_handler(handler);

        Ok(lib)
    }
}

fn get_handler(config: &ServerConfig) -> Result<(Option<Library>, Handler)> {
    let mut handler = Handler::default();
    let mut _lib = None;

    if let Some(path) = &config.setup_lib {
        let mut handler = Handler::new();
        _lib = Some(load_lib(&mut handler, path)?);
        return Ok((_lib, handler));
    }

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
        |req: &mut HttpRequest| {
            use std::process::{Command, Stdio};

            let output = Command::new("php")
                .arg(&*req.filename().unwrap())
                .stdout(Stdio::piped())
                .spawn()?
                .wait_with_output()?;

            req.respond_buf(&output.stdout)
        },
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

    Ok((_lib, handler))
}

pub fn main() {
    let args: Vec<_> = env::args().skip(1).collect();
    let config = ServerConfig::parse(&args).unwrap_or_else(|err| {
        eprintln!("{err}");
        process::exit(1);
    });

    let (_lib, handler) = get_handler(&config).unwrap_or_else(|err| {
        eprintln!("ERROR: {err}");
        process::exit(1);
    });

    let mut server = HttpServer::new(config).unwrap_or_else(|err| {
        eprintln!("ERROR: {err}");
        std::process::exit(1)
    });

    server.set_handler(handler);
    server.run();
}
