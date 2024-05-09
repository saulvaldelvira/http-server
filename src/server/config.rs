use std::{env, path::Path, str::FromStr, time::Duration};

#[derive(Clone, Copy)]
pub struct ServerConfig {
    port: u16,
    n_threads: usize,
    keep_alive_timeout: Duration,
    keep_alive_requests: u16,
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

/// [crate::HttpServer] configuration
///
/// # Example
/// ```
/// use http_server::server::ServerConfig;
///
/// let mut conf = ServerConfig::default();
/// conf.set_port(8080)
///     .set_n_threads(1024);
/// ```
impl ServerConfig {
    /// Default configuration
    pub fn default() -> Self {
        Self {
            port: 80,
            n_threads: 1024,
            keep_alive_timeout: Duration::from_secs(0),
            keep_alive_requests: 10000,
        }
    }
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
                "-n" => conf.n_threads = parse_next!("-n"),
                "-d" | "--dir" => {
                    let path:String = parse_next!(arg.as_str());
                    env::set_current_dir(&Path::new(&path)).expect("Error changing cwd");
                },
                "--keep-alive" => {
                    let timeout = parse_next!("--keep-alive");
                     conf.keep_alive_timeout = Duration::from_secs_f32(timeout);
                },
                "--keep-alive-requests" => conf.keep_alive_requests = parse_next!("--keep-alive-requests"),
                _ => panic!("Unknow argument: {arg}")
            }
        }
        conf
    }
    pub fn port(&self) -> u16 { self.port }
    pub fn n_threads(&self) -> usize { self.n_threads }
    pub fn keep_alive(&self) -> bool { self.keep_alive_timeout.as_millis() > 0 }
    pub fn keep_alive_timeout(&self) -> Duration { self.keep_alive_timeout }
    pub fn keep_alive_requests(&self) -> u16 { self.keep_alive_requests }
    pub fn set_port(&mut self, port:u16) -> &mut Self {
        self.port = port;
        self
    }
    pub fn set_n_threads(&mut self, n: usize) -> &mut Self {
        self.n_threads = n;
        self
    }
    pub fn set_keep_alive_timeout(&mut self, timeout: Duration) -> &mut Self {
        self.keep_alive_timeout = timeout;
        self
    }
    pub fn set_keep_alive_requests(&mut self, n: u16) -> &mut Self {
        self.keep_alive_requests = n;
        self
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
