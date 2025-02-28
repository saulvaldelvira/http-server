#![allow(clippy::cast_sign_loss, clippy::cast_possible_truncation)]

use std::{
    env, fs,
    path::{Path, PathBuf},
    process,
    str::FromStr,
    time::Duration,
};

use jsonrs::Json;
use pool::PoolConfig;

use crate::{
    Result,
    log::{self},
    log_info, log_warn,
};

#[derive(Clone, Debug)]
pub struct ServerConfig {
    pub port: u16,
    pub pool_conf: PoolConfig,
    pub keep_alive_timeout: Duration,
    pub keep_alive_requests: u16,
    pub log_file: Option<String>,
}

#[cfg(not(test))]
fn get_default_conf_file() -> Option<PathBuf> {
    if let Ok(path) = env::var("XDG_CONFIG_HOME") {
        let mut p = PathBuf::new();
        p.push(path);
        p.push("http-srv");
        p.push("config.json");
        Some(p)
    } else if let Ok(path) = env::var("HOME") {
        let mut p = PathBuf::new();
        p.push(path);
        p.push(".config");
        p.push("http-srv");
        p.push("config.json");
        Some(p)
    } else {
        None
    }
}

#[cfg(test)]
fn get_default_conf_file() -> Option<PathBuf> {
    None
}

/// [`crate::HttpServer`] configuration
///
/// # Example
/// ```
/// use http_srv::ServerConfig;
/// use pool::PoolConfig;
///
/// let pool_conf = PoolConfig::builder()
///                 .n_workers(120_u16)
///                 .build().unwrap();
/// let conf =
/// ServerConfig::default()
///     .port(8080)
///     .pool_config(pool_conf);
/// ```
impl ServerConfig {
    /// Parse the configuration from the command line args
    pub fn parse<S: AsRef<str>>(args: &[S]) -> Result<Self> {
        let mut conf = Self::default();

        let mut conf_file = get_default_conf_file();

        /* Parse the --config-file before the rest */
        let mut first_pass = args.iter();
        while let Some(arg) = first_pass.next() {
            if arg.as_ref() == "--config-file" {
                if let Some(fname) = first_pass.next() {
                    let filename = PathBuf::from(fname.as_ref());
                    if filename.exists() {
                        conf_file = Some(filename);
                    } else {
                        log_warn!(
                            "Config path: {} doesn't exist",
                            filename.as_os_str().to_str().unwrap_or("[??]")
                        );
                    }
                }
            }
        }

        if let Some(cfile) = conf_file {
            conf.parse_conf_file(&cfile)?;
        }

        let mut args = args.iter();
        while let Some(arg) = args.next() {
            macro_rules! parse_next {
                () => {
                    args.next_parse().ok_or_else(|| {
                        format!("Missing or incorrect argument for \"{}\"", arg.as_ref())
                    })?
                };
                (as $t:ty) => {{
                    let _next: $t = parse_next!();
                    _next
                }};
            }

            let mut pool_conf_builder = PoolConfig::builder();

            match arg.as_ref() {
                "-p" | "--port" => conf.port = parse_next!(),
                "-n" | "-n-workers" => {
                    pool_conf_builder.n_workers(parse_next!(as u16));
                }
                "-d" | "--dir" => {
                    let path: String = parse_next!();
                    env::set_current_dir(Path::new(&path))?;
                }
                "-k" | "--keep-alive" => {
                    let timeout = parse_next!();
                    conf.keep_alive_timeout = Duration::from_secs_f32(timeout);
                }
                "-r" | "--keep-alive-requests" => conf.keep_alive_requests = parse_next!(),
                "-l" | "--log" => conf.log_file = Some(parse_next!()),
                "--log-level" => {
                    let n: u8 = parse_next!();
                    log::set_level(n.try_into()?);
                }
                "--config-file" => {}
                "-h" | "--help" => help(),
                unknown => return Err(format!("Unknow argument: {unknown}").into()),
            }
        }

        log_info!("{conf:#?}");
        Ok(conf)
    }
    fn parse_conf_file(&mut self, conf_file: &Path) -> crate::Result<()> {
        if !conf_file.exists() {
            return Ok(());
        }
        let conf_str = conf_file.as_os_str().to_str().unwrap_or("");
        let f = fs::read_to_string(conf_file).unwrap_or_else(|err| {
            eprintln!("Error reading config file \"{conf_str}\": {err}");
            std::process::exit(1);
        });
        let json = Json::deserialize(&f).unwrap_or_else(|err| {
            eprintln!("Error parsing config file: {err}");
            std::process::exit(1);
        });
        log_info!("Parsing config file: {conf_str}");
        let Json::Object(obj) = json else {
            return Err("Expected json object".into());
        };
        for (k, v) in obj {
            macro_rules! num {
                () => {
                    num!(v)
                };
                ($v:ident) => {
                    $v.number().ok_or_else(|| {
                        format!("Parsing config file ({conf_str}): Expected number for \"{k}\"")
                    })?
                };
                ($v:ident as $t:ty) => {{
                    let _n = num!($v);
                    _n as $t
                }};
            }
            macro_rules! string {
                () => {
                    v.string()
                        .ok_or_else(|| {
                            format!("Parsing config file ({conf_str}): Expected string for \"{k}\"")
                        })?
                        .to_string()
                };
            }
            macro_rules! obj {
                () => {
                    v.object().ok_or_else(|| {
                        format!("Parsing config file ({conf_str}): Expected object for \"{k}\"")
                    })?
                };
            }

            match &*k {
                "port" => self.port = num!() as u16,
                "root_dir" => {
                    let path: String = string!();
                    let path = path.replacen(
                        '~',
                        env::var("HOME").as_ref().map(String::as_str).unwrap_or("~"),
                        1,
                    );
                    env::set_current_dir(Path::new(&path))?;
                }
                "keep_alive_timeout" => self.keep_alive_timeout = Duration::from_secs_f64(num!()),
                "keep_alive_requests" => self.keep_alive_requests = num!() as u16,
                "log_file" => self.log_file = Some(string!()),
                "log_level" => {
                    let n = num!(v as u8);
                    log::set_level(n.try_into()?);
                }
                "pool_config" => {
                    for (k, v) in obj!() {
                        match &**k {
                            "n_workers" => self.pool_conf.n_workers = num!(v as u16),
                            "pending_buffer_size" => {
                                let n = v.number().map(|n| n as u16);
                                self.pool_conf.incoming_buf_size = n;
                            }
                            _ => log_warn!(
                                "Parsing config file ({conf_str}): Unexpected key: \"{k}\""
                            ),
                        }
                    }
                }
                _ => log_warn!("Parsing config file ({conf_str}): Unexpected key: \"{k}\""),
            };
        }
        Ok(())
    }
    #[inline]
    #[must_use]
    pub fn pool_config(mut self, conf: PoolConfig) -> Self {
        self.pool_conf = conf;
        self
    }
    #[inline]
    #[must_use]
    pub fn port(mut self, port: u16) -> Self {
        self.port = port;
        self
    }
    #[inline]
    #[must_use]
    pub fn keep_alive_timeout(mut self, timeout: Duration) -> Self {
        self.keep_alive_timeout = timeout;
        self
    }
    #[inline]
    #[must_use]
    pub fn keep_alive_requests(mut self, n: u16) -> Self {
        self.keep_alive_requests = n;
        self
    }
}

fn help() -> ! {
    println!(
        "\
USAGE: http-srv [-p <port>] [-n <n-workers>] [-d <working-dir>]
PARAMETERS:
    -p, --port <port>    TCP Port to listen for requests
    -n, --n-workers <n>  Number of concurrent workers
    -d, --dir <working-dir>  Root directory of the server
    -k, --keep-alive <sec>   Keep alive seconds
    -r, --keep-alive-requests <num>  Keep alive max requests
    -l, --log <file>   Set log file
    --log-level <n>    Set log level
    -h, --help  Display this help message
    --conf <file> Use the given config file instead of the default one
EXAMPLES:
  http-srv -p 8080 -d /var/html
  http-srv -d ~/desktop -n 1024 --keep-alive 120
  http-srv --log /var/log/http-srv.log"
    );
    process::exit(0);
}

trait ParseIterator {
    fn next_parse<T: FromStr>(&mut self) -> Option<T>;
}

impl<I, R: AsRef<str>> ParseIterator for I
where
    I: Iterator<Item = R>,
{
    fn next_parse<T: FromStr>(&mut self) -> Option<T> {
        self.next()?.as_ref().parse().ok()
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
            pool_conf: PoolConfig::default(),
            keep_alive_timeout: Duration::from_secs(0),
            keep_alive_requests: 10000,
            log_file: None,
        }
    }
}

#[cfg(test)]
mod test {
    #![allow(clippy::unwrap_used)]

    use crate::ServerConfig;

    #[test]
    fn valid_args() {
        let conf = vec!["-p".to_string(), "80".to_string()];
        ServerConfig::parse(&conf).unwrap();
    }

    macro_rules! expect_err {
        ($conf:expr , $msg:literal) => {
            match ServerConfig::parse(&$conf) {
                Ok(c) => panic!("Didn't panic: {c:#?}"),
                Err(msg) => assert_eq!(msg.get_message(), $msg),
            }
        };
    }

    #[test]
    fn unknown() {
        let conf = vec!["?"];
        expect_err!(conf, "Unknow argument: ?");
    }

    #[test]
    fn missing() {
        let conf = vec!["-n"];
        expect_err!(conf, "Missing or incorrect argument for \"-n\"");
    }

    #[test]
    fn parse_error() {
        let conf = vec!["-p", "abc"];
        expect_err!(conf, "Missing or incorrect argument for \"-p\"");
    }
}
