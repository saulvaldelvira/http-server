use std::{
    env,
    fs::File,
    io,
    io::{Write, stdout},
    net::{TcpStream, ToSocketAddrs},
    process,
};

use http::{HttpMethod, HttpRequest, HttpResponse};
mod config;
use config::ClientConfig;

use crate::config::HttpType;

fn open_file(fname: &str) -> Box<dyn Write> {
    Box::new(File::create(fname).unwrap_or_else(|_| {
        eprintln!("Couldn't create file: {fname}");
        process::exit(1);
    }))
}

#[cfg(not(feature = "tls"))]
#[inline]
fn send_request(
    tcp: TcpStream,
    _http_type: HttpType,
    _host: String,
    req: HttpRequest,
) -> http::Result<HttpResponse> {
    req.send_to(tcp)
}

#[cfg(feature = "tls")]
fn send_request(
    tcp: TcpStream,
    http_type: HttpType,
    host: String,
    req: HttpRequest,
) -> http::Result<HttpResponse> {
    use std::sync::Arc;

    if matches!(http_type, HttpType::Https) {
        let root_store = rustls::RootCertStore {
            roots: webpki_roots::TLS_SERVER_ROOTS.into(),
        };
        let mut config = rustls::ClientConfig::builder()
            .with_root_certificates(root_store)
            .with_no_client_auth();

        config.key_log = Arc::new(rustls::KeyLogFile::new());

        let conn =
            rustls::ClientConnection::new(Arc::new(config), host.try_into().unwrap()).unwrap();
        let tls = rustls::StreamOwned::new(conn, tcp);
        req.send_to(tls)
    } else {
        req.send_to(tcp)
    }
}

pub fn main() -> http::Result<()> {
    let conf = ClientConfig::parse(env::args().skip(1)).unwrap_or_else(|err| {
        eprintln!("ERROR: {err}");
        std::process::exit(1)
    });

    let addr = format!("{}:{}", conf.host, conf.port);
    let addrs = addr.to_socket_addrs().unwrap().next().unwrap();

    let mut out: Box<dyn Write> = match conf.out_file {
        config::OutFile::Stdout => Box::new(stdout()),
        config::OutFile::Filename(s) => open_file(&s),
        config::OutFile::GetFromUrl => {
            let fname = conf
                .url
                .split('/')
                .filter(|s| !s.is_empty())
                .next_back()
                .unwrap_or(&conf.host);
            open_file(fname)
        }
    };

    let req = HttpRequest::builder()
        .method(conf.method)
        .url(conf.url.clone().into_boxed_str())
        .version(1.1)
        .header("Host", conf.host.clone().into_boxed_str())
        .header("Accept", "*/*")
        .header("User-Agent", "http-client")
        .header("Connection", "close")
        .header("Accept-Encoding", "identity")
        .build()
        .unwrap();

    let tcp = match TcpStream::connect(addrs) {
        Ok(tcp) => tcp,
        Err(e) => {
            eprintln!("Error connecting to {addrs}: {e}");
            process::exit(1);
        }
    };

    let mut result =
        send_request(tcp, conf.http_type, conf.host.clone(), req).unwrap_or_else(|err| {
            eprint!("ERROR: {err}");
            process::exit(1);
        });

    if matches!(conf.method, HttpMethod::HEAD) {
        println!("Headers");
        for (k, v) in result.headers() {
            println!("{k}: {v}");
        }
        return Ok(());
    }

    match result.write_to(&mut out) {
        Ok(_) => { /* eprintln!("\n\n{n} bytes transfered") */ }
        Err(err) => {
            match err.kind() {
                /* Ignore this error kind
                 * https://docs.rs/rustls/latest/rustls/manual/_03_howto/index.html#unexpected-eof
                 * */
                io::ErrorKind::UnexpectedEof => {}
                _ => eprintln!("\n\nERROR: {err}"),
            }
        }
    }

    Ok(())
}
