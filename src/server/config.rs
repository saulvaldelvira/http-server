use std::{env, fs, path::{Path, PathBuf}, process, str::FromStr, time::Duration};
use crate::{log::{self}, log_info, log_warn, Result};
use jsonrs::Json;

#[derive(Clone)]
pub struct ServerConfig {
    pub port: u16,
    pub n_workers: u16,
    pub keep_alive_timeout: Duration,
    pub keep_alive_requests: u16,
    pub log_file: Option<String>,
}

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
    pub fn parse<I>(args: I) -> Result<Self>
    where I: Iterator<Item = String>
    {
        let mut conf = Self::default();
        let mut args = args.into_iter();

        let mut config_file: Option<PathBuf> = None;

        while let Some(arg) = args.next() {
            macro_rules! parse_next {
                () => {
                    args.next_parse().ok_or_else(|| {
                        format!("Missing or incorrect argument for \"{}\"", arg.as_str())
                    })?
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
                "--log-level" => {
                    let n : u8 = parse_next!();
                    log::set_level(n.into())
                }
                "--config-file" => config_file = Some( parse_next!() ),
                "-h" | "--help" => help(),
                _ => return Err(format!("Unknow argument: {arg}").into())
            }

        }
        let mut must_exist = true;
        if config_file.is_none() {
            config_file = get_default_conf_file();
            must_exist = false;
        }
        if let Some(file) = &config_file {
            if file.exists() {
                conf.parse_conf_file(file.as_path())?;
            } else if must_exist {
                log_warn!("Config path: {} doesn't exist", file.as_os_str().to_str().unwrap_or("[??]"));
            }
        }
        Ok(conf)
    }
    fn parse_conf_file(&mut self, conf_file: &Path) -> crate::Result<()> {
        if !conf_file.exists() { return Ok(()) }
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
        let Json::Object(obj) = json else { return Err("Expected json object".into()) };
        for (k,v) in obj {
            macro_rules! num {
                () => {
                    v.number().ok_or_else(|| format!("Parsing config file ({conf_str}): Expected number for \"{k}\""))?
                };
            }
            macro_rules! string {
                () => {
                    v.string().ok_or_else(|| format!("Parsing config file ({conf_str}): Expected stirng for \"{k}\""))?.to_string()
                };
            }
            macro_rules! mch {
                ($( $k:literal => $action:expr  ),*) => {
                    match k.as_str() {
                        $(
                            $k => {
                                $action ;
                                log_info!("Override {} with {v}", $k);
                            }
                        ),*
                        _ => log_info!("Parsing config file ({conf_str}): Unexpected key: \"{k}\""),
                    }
                };
            }
            mch! {
                "port" => self.port = num!() as u16,
                "n_workers" => self.n_workers = num!() as u16,
                "root_dir" => {
                    let path:String = string!();
                    let path = path.replacen('~', env::var("HOME").as_ref().map(|s| s.as_str()).unwrap_or("~"), 1);
                    env::set_current_dir(Path::new(&path)).expect("Error changing cwd");
                },
                "keep_alive_timeout" => self.keep_alive_timeout = Duration::from_secs_f64(num!()),
                "keep_alive_requests" => self.keep_alive_requests = num!() as u16,
                "log_file" => self.log_file = Some( string!() )
            };
        }
        Ok(())
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
    use crate::{ServerConfig,Result};

    fn parse_from_vec(v: Vec<&str>) -> Result<ServerConfig> {
        let conf = v.iter().map(|s| s.to_string());
        ServerConfig::parse(conf)
    }

    #[test]
    fn valid_args() {
        let conf = vec!["-p", "80"];
        parse_from_vec(conf).unwrap();
    }

    macro_rules! expect_err {
        ($conf:expr , $msg:literal) => {
            let Err(msg) = parse_from_vec($conf) else { panic!() };
            assert_eq!(msg.get_message(), $msg);
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
        expect_err!(conf,"Missing or incorrect argument for \"-n\"");
    }

    #[test]
    fn parse_error() {
        let conf = vec!["-p","abc"];
        expect_err!(conf,"Missing or incorrect argument for \"-p\"");
    }
}
