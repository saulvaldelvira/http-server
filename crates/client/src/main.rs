use std::{
    env,
    fs::File,
    io::{stdout, Write},
    net::{TcpStream, ToSocketAddrs},
    process,
};

use http::{HttpMethod, HttpRequest, HttpStream};
mod config;
use config::ClientConfig;

fn open_file(fname: &str) -> Box<dyn Write> {
    Box::new(File::create(fname).unwrap_or_else(|_| {
        eprintln!("Couldn't create file: {fname}");
        process::exit(1);
    }))
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
                .last()
                .unwrap_or(&conf.host);
            open_file(fname)
        }
    };

    let req = HttpRequest::builder()
        .method(HttpMethod::GET)
        .url(conf.url)
        .version(1.1)
        .header("Host", conf.host)
        .header("Accept", "*/*")
        .header("User-Agent", "http-client")
        .build()
        .unwrap();

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
        Ok(_) => { /* eprintln!("\n\n{n} bytes transfered") */ }
        Err(err) => eprintln!("\n\nERROR: {err}"),
    }

    Ok(())
}
