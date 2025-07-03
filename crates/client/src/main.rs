use std::{
    env,
    fs::File,
    io,
    io::{Write, stdout},
    net::{TcpStream, ToSocketAddrs},
    process,
    time::Instant,
};

use http::{HttpMethod, HttpRequest, HttpResponse};
mod config;
use config::ClientConfig;

use crate::config::HttpType;

pub struct ProgressWriter<W: Write> {
    max: usize,
    current: usize,
    last_update: Instant,
    prev_current: usize,
    writer: W,
}

impl<W: Write> ProgressWriter<W> {
    fn start(&self) {
        println!(
            "{perc:3}  {progress:8}  {total:^8}  speed",
            perc = "%",
            progress = "Progress",
            total = "Total",
        );
    }

    fn print_progress(&self) {
        let Self { max, current, .. } = self;

        fn get_unit(bytes: usize) -> String {
            let kb = bytes / 1000;
            if kb >= 1000 {
                format!("{} M", kb / 1000)
            } else {
                format!("{kb} K")
            }
        }

        let curr = get_unit(*current);
        let max = get_unit(*max);

        if self.max > 0 {
            let percentage = 100 * self.current / self.max;
            print!("\x1b[2K\r{percentage:<3}  {curr:^8}  {max:^8}  ");
        } else {
            print!("\x1b[2K\r{current}");
        }
        let kbps = (self.current - self.prev_current) / 1000;
        if kbps >= 1000 {
            print!("{} Mb/s", kbps / 1000);
        } else {
            print!("{kbps} Kb/s");
        }
        stdout().flush().unwrap();
    }
}

impl<W: Write> Write for ProgressWriter<W> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let n = self.writer.write(buf)?;
        self.current += n;

        let time = Instant::now().duration_since(self.last_update);
        if time.as_millis() >= 1000 {
            self.print_progress();
            self.last_update = Instant::now();
            self.prev_current = self.current;
        }

        Ok(n)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.writer.flush()
    }
}

fn open_file(fname: &str, max: usize) -> Box<dyn Write> {
    let file = File::create(fname).unwrap_or_else(|_| {
        eprintln!("Couldn't create file: {fname}");
        process::exit(1);
    });
    let p = Box::new(ProgressWriter {
        max,
        current: 0,
        prev_current: 0,
        writer: file,
        last_update: Instant::now(),
    });
    p.start();
    p
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

    let req = HttpRequest::builder()
        .method(conf.method)
        .url(conf.url.clone().into_boxed_str())
        .version(1.1)
        .header("Host", conf.host.clone().into_boxed_str())
        .header("Accept", "*/*")
        .header("User-Agent", "http-client")
        .header("Connection", "close")
        .header("Accept-Encoding", "identity")
        .build();

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

    let len = result.content_length();
    let mut out: Box<dyn Write> = match conf.out_file {
        config::OutFile::Stdout => Box::new(stdout()),
        config::OutFile::Filename(s) => open_file(&s, len),
        config::OutFile::GetFromUrl => {
            let fname = conf
                .url
                .split('/')
                .filter(|s| !s.is_empty())
                .next_back()
                .unwrap_or(&conf.host);
            open_file(fname, len)
        }
    };

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
