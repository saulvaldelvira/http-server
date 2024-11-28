use std::{process, str::FromStr};
use crate::{HttpMethod, Result};

#[derive(Debug,Clone)]
pub enum OutFile {
    Stdout,
    Filename(String),
    GetFromUrl,
}

#[derive(Clone,Debug)]
pub struct ClientConfig {
    pub url: String,
    pub method: HttpMethod,
    pub host: String,
    pub user_agent: String,
    pub out_file: OutFile,
}

impl ClientConfig {
    /// Parse the configuration from the command line args
    pub fn parse<I>(args: I) -> Result<Self>
    where I: Iterator<Item = String>
    {
        let mut conf = Self::default();
        let mut args = args.into_iter();

        while let Some(arg) = args.next() {
            macro_rules! parse_next {
                () => {
                    args.next_parse().ok_or_else(|| {
                        format!("Missing or incorrect argument for \"{}\"", arg.as_str())
                    })?
                };
                (as $t:ty) => {
                    {
                        let _next: $t = parse_next!();
                        _next
                    }
                };
            }

            match arg.as_str() {
                "-m" | "--method" => conf.method = parse_next!(),
                "--host" => conf.host = parse_next!(),
                "-a" | "--user-agent" => conf.user_agent = parse_next!(),
                "-h" | "--help" => help(),
                "-O" => conf.out_file = OutFile::GetFromUrl,
                "-o" => conf.out_file = OutFile::Filename(parse_next!()),
                _ => conf.url = arg
            }
        }

        let mut host = conf.url.as_str();
        host = conf.url.strip_prefix("http://").unwrap_or(host);

        let mut url = "/";

        if let Some(i) = host.find("/") {
            url = &host[i..];
            host = &host[..i];
        }

        conf.host = host.to_string();
        conf.url = url.to_string();

        Ok(conf)
    }
}

fn help() -> ! {
    println!("\
USAGE: http-srv [... PARAMS ...] url<:port>
PARAMETERS:
    -m, --method Set HTTP method
    --host Set host
    -a, --user-agent Set user agent
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

impl Default for ClientConfig {
    /// Default configuration
    ///
    /// - Port: 80
    /// - Nº Workers: 1024
    /// - Keep Alive Timeout: 0s (Disabled)
    /// - Keep Alove Requests: 10000
    #[inline]
    fn default() -> Self {
        Self {
            method: HttpMethod::GET,
            url: "".to_string(),
            host: "".to_string(),
            user_agent: "http-client".to_string(),
            out_file: OutFile::Stdout
        }
    }
}

/* #[cfg(test)] */
/* mod test { */
/*     use crate::{ServerConfig,Result}; */

/*     fn parse_from_vec(v: Vec<&str>) -> Result<ServerConfig> { */
/*         let conf = v.iter().map(|s| s.to_string()); */
/*         ServerConfig::parse(conf) */
/*     } */

/*     #[test] */
/*     fn valid_args() { */
/*         let conf = vec!["-p", "80"]; */
/*         parse_from_vec(conf).unwrap(); */
/*     } */

/*     macro_rules! expect_err { */
/*         ($conf:expr , $msg:literal) => { */
/*             let Err(msg) = parse_from_vec($conf) else { panic!() }; */
/*             assert_eq!(msg.get_message(), $msg); */
/*         }; */
/*     } */

/*     #[test] */
/*     fn unknown() { */
/*         let conf = vec!["?"]; */
/*         expect_err!(conf, "Unknow argument: ?"); */
/*     } */

/*     #[test] */
/*     fn missing() { */
/*         let conf = vec!["-n"]; */
/*         expect_err!(conf,"Missing or incorrect argument for \"-n\""); */
/*     } */

/*     #[test] */
/*     fn parse_error() { */
/*         let conf = vec!["-p","abc"]; */
/*         expect_err!(conf,"Missing or incorrect argument for \"-p\""); */
/*     } */
/* } */
