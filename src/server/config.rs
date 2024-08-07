use std::{env, path::Path, process, str::FromStr, time::Duration};

#[derive(Clone)]
pub struct ServerConfig {
    pub port: u16,
    pub n_workers: u16,
    pub keep_alive_timeout: Duration,
    pub keep_alive_requests: u16,
    pub log_file: Option<String>,
}

/// [crate::HttpServer] configuration
///
/// # Example
/// ```
/// use http_srv::server::ServerConfig;
///
/// let mut conf =
/// ServerConfig::default()
///     .port(8080)
///     .n_workers(1024);
/// ```
impl ServerConfig {
    /// Parse the configuration from the command line args
    pub fn parse<I>(args: I) -> Self
    where I: Iterator<Item = String>
    {
        let mut conf = Self::default();
        let mut args = args.into_iter();

        while let Some(arg) = args.next() {
            macro_rules! parse_next {
                () => {
                    args.next_parse().unwrap_or_else(|| {
                        eprintln!("Missing or incorrect argument for \"{}\"", arg.as_str());
                        std::process::exit(1);
                    })
                };
            }

            match arg.as_str() {
                "-p" | "--port" => conf.port = parse_next!(),
                "-n" | "-n-workers" => conf.n_workers = parse_next!(),
                "-d" | "--dir" => {
                    let path:String = parse_next!();
                    env::set_current_dir(Path::new(&path)).expect("Error changing cwd");
                },
                "-k" | "--keep-alive" => {
                    let timeout = parse_next!();
                     conf.keep_alive_timeout = Duration::from_secs_f32(timeout);
                },
                "-r" | "--keep-alive-requests" => conf.keep_alive_requests = parse_next!(),
                "-l" | "--log" => conf.log_file = Some( parse_next!() ),
                "-h" | "--help" => help(),
                _ => panic!("Unknow argument: {arg}")
            }
        }
        conf
    }
    #[inline]
    pub fn n_workers(mut self, n_workers: u16) -> Self {
        self.n_workers = n_workers;
        self
    }
    #[inline]
    pub fn port(mut self, port:u16) -> Self {
        self.port = port;
        self
    }
    #[inline]
    pub fn keep_alive_timeout(mut self, timeout: Duration) -> Self {
        self.keep_alive_timeout = timeout;
        self
    }
    #[inline]
    pub fn keep_alive_requests(mut self, n: u16) -> Self {
        self.keep_alive_requests = n;
        self
    }
}

fn help() -> ! {
    println!("\
USAGE: http-srv [-p <port>] [-n <n-workers>] [-d <working-dir>]
PARAMETERS:
    -p, --port <port>    TCP Port to listen for requests
    -n, --n-workers <n>  Number of concurrent workers
    -d, --dir <working-dir>  Root directory of the server
    -k, --keep-alive <sec>   Keep alive seconds
    -r, --keep-alive-requests <num>  Keep alive max requests
    -l, --log <file>   Set log file
    -h, --help  Display this help message
EXAMPLES:
  http-srv -p 8080 -d /var/html
  http-srv -d ~/desktop -n 1024 --keep-alive 120
  http-srv --log /var/log/http-srv.log");
    process::exit(0);
}

trait ParseIterator {
    fn next_parse<T: FromStr>(&mut self) -> Option<T>;
}

impl<I> ParseIterator for I
where I: Iterator<Item = String>
{
    fn next_parse<T: FromStr>(&mut self) -> Option<T> {
        self.next()?.parse().ok()
    }
}

impl Default for ServerConfig {
    /// Default configuration
    ///
    /// - Port: 80
    /// - Nº Workers: 1024
    /// - Keep Alive Timeout: 0s (Disabled)
    /// - Keep Alove Requests: 10000
    #[inline]
    fn default() -> Self {
        Self {
            port: 80,
            n_workers: 1024,
            keep_alive_timeout: Duration::from_secs(0),
            keep_alive_requests: 10000,
            log_file: None,
        }
    }
}

#[cfg(test)]
mod test {
    use crate::ServerConfig;

    fn parse_from_vec(v: Vec<&str>) {
        let conf = v.iter().map(|s| s.to_string());
        ServerConfig::parse(conf);
    }

    #[test]
    fn valid_args() {
        let conf = vec!["-p", "80"];
        parse_from_vec(conf);
    }

    #[test]
    #[should_panic(expected = "Unknow argument: ?")]
    fn unknown() {
        let conf = vec!["?"];
        parse_from_vec(conf);
    }

    #[test]
    #[should_panic(expected = "Missing argument for -n")]
    fn missing() {
        let conf = vec!["-n"];
        parse_from_vec(conf);
    }

    #[test]
    #[should_panic(expected = "Missing argument for -p")]
    fn parse_error() {
        let conf = vec!["-p","abc"];
        parse_from_vec(conf);
    }
}
