use std::{env, path::Path, process, str::FromStr, time::Duration};

#[derive(Clone, Copy)]
pub struct ServerConfig {
    pub port: u16,
    pub n_workers: u16,
    pub keep_alive_timeout: Duration,
    pub keep_alive_requests: u16,
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

        macro_rules! parse_next {
            ($arg:expr) => {
                args.next_parse().expect(&format!("Missing argument for {}", $arg))
            };
        }

        while let Some(arg) = args.next() {
            match arg.as_str() {
                "-p" => conf.port = parse_next!("-p"),
                "-n" => conf.n_workers = parse_next!("-n"),
                "-d" | "--dir" => {
                    let path:String = parse_next!(arg.as_str());
                    env::set_current_dir(&Path::new(&path)).expect("Error changing cwd");
                },
                "--keep-alive" => {
                    let timeout = parse_next!("--keep-alive");
                     conf.keep_alive_timeout = Duration::from_secs_f32(timeout);
                },
                "--keep-alive-requests" => conf.keep_alive_requests = parse_next!("--keep-alive-requests"),
                "-h" | "--help" => help(),
                _ => panic!("Unknow argument: {arg}")
            }
        }
        conf
    }
    #[inline]
    pub fn keep_alive(&self) -> bool { self.keep_alive_timeout.as_millis() > 0 }
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

fn help() {
    println!("\
USAGE: http-server [-p <port>] [-n <n-workers>] [-d <working-dir>]
PARAMETERS:
    -p <port> : TCP Port to listen for requests
    -n <n-workers> : Number of concurrent workers
    -d <working-dir> : Root directory of the server
    -h | --help : Display this help message");
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
        }
    }
}
