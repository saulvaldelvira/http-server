use std::fs::File;
use std::io::Write;
use std::{env, io::stdout, net::{TcpStream, ToSocketAddrs}, process};
use http_srv::client::{config, ClientConfig};

use http_srv::http::HttpStream;
use http_srv::{http::HttpMethod, request::HttpRequest};

fn open_file(fname: &str) -> Box<dyn Write> {
    Box::new(File::create(fname).unwrap_or_else(|_| {
        eprintln!("Couldn't create file: {fname}");
        process::exit(1);
    }))
}

pub fn main() -> http_srv::Result<()> {
    let conf = ClientConfig::parse(env::args().skip(1)).unwrap();

    let addrs = conf.host.to_socket_addrs().unwrap().next().unwrap();

    let mut out: Box<dyn Write> = match conf.out_file {
        config::OutFile::Stdout => Box::new(stdout()) ,
        config::OutFile::Filename(s) => open_file(&s),
        config::OutFile::GetFromUrl => {
            let fname = conf.url.split('/').filter(|s| !s.is_empty()).last().unwrap_or(&conf.host);
            open_file(fname)
        },
    };

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
    let tcp = HttpStream::from(tcp);
    let mut result = req.send_to(tcp).unwrap_or_else(|err| {
        eprint!("ERROR: {err}");
        process::exit(1);
    });

    match result.write_to(&mut out) {
        Ok(_) => {/* eprintln!("\n\n{n} bytes transfered") */},
        Err(err) => eprint!("\n\nERROR: {err}")
    }

    Ok(())
}
