use core::num::ParseIntError;
use std::net::{TcpStream, ToSocketAddrs};

pub use http::{self, HttpRequest, HttpResponse};

#[derive(Clone, Debug)]
pub enum HttpType {
    Http,
    Https,
}

pub fn parse_url(url: &str) -> Result<(HttpType, i32, &str, &str), ParseIntError> {
    let mut host = url;
    let mut http_type = HttpType::Http;

    if let Some(url) = url.strip_prefix("http://") {
        host = url;
    }
    if let Some(url) = url.strip_prefix("https://") {
        host = url;
        http_type = HttpType::Https;
    }

    let mut url = "/";

    if let Some(i) = host.find('/') {
        url = &host[i..];
        host = &host[..i];
    }

    let port;
    if let Some(i) = host.find(':') {
        port = host[i + 1..].parse()?;
        host = &host[..i];
    } else {
        port = match http_type {
            HttpType::Http => 80,
            HttpType::Https => 443,
        }
    }

    Ok((http_type, port, host, url))
}

pub fn send_request(
    req: HttpRequest,
    host: &str,
    port: u16,
    http_type: HttpType,
) -> http::Result<HttpResponse> {
    let addr = format!("{}:{}", host, port);
    let addrs = addr
        .to_socket_addrs()?
        .next()
        .ok_or_else(|| http::HttpError::new(format!("No address for \"{host}\"")))?;

    let tcp = TcpStream::connect(addrs)?;

    #[cfg(feature = "tls")]
    {
        use std::sync::Arc;
        if matches!(http_type, HttpType::Https) {
            let root_store = rustls::RootCertStore {
                roots: webpki_roots::TLS_SERVER_ROOTS.into(),
            };
            let mut config = rustls::ClientConfig::builder()
                .with_root_certificates(root_store)
                .with_no_client_auth();

            config.key_log = Arc::new(rustls::KeyLogFile::new());

            let conn = rustls::ClientConnection::new(
                Arc::new(config),
                host.to_owned().try_into().unwrap(),
            )
            .unwrap();
            let tls = rustls::StreamOwned::new(conn, tcp);
            return req.send_to(tls);
        }
    }

    #[cfg(not(feature = "tls"))]
    if matches!(http_type, HttpType::Https) {
        return Err("HTTPS not supported".into());
    }

    req.send_to(tcp)
}
