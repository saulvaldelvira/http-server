use std::{env, io::stdout, net::{TcpStream, ToSocketAddrs}, process};
use super::ClientConfig;

use crate::{http::HttpMethod, request::HttpRequest};

pub fn main() {
    let conf = ClientConfig::parse(env::args().skip(1)).unwrap();

    let addrs = conf.host.to_socket_addrs().unwrap().next().unwrap();

    let req = HttpRequest::builder()
                .method(HttpMethod::GET)
                .url(conf.url)
                .version(1.1)
                .header("Host", conf.host)
                .header("Accept", "*/*")
                .header("User-Agent", "http-client")
                .build().unwrap();
    let tcp = match TcpStream::connect(addrs) {
        Ok(tcp) => tcp,
        Err(e) => {
            eprintln!("Error connecting to {addrs}: {e}");
            process::exit(1);
        }
    };
    let mut result = req.send_to(tcp).unwrap();

    match result.write_to(&mut stdout()) {
        Ok(_) => {/* eprintln!("\n\n{n} bytes transfered") */},
        Err(err) => eprint!("\n\nERROR: {err}")
    }
}
