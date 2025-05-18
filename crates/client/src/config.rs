use std::{process, str::FromStr};

use http::{HttpMethod, Result};

#[derive(Debug, Clone)]
pub enum OutFile {
    Stdout,
    Filename(String),
    GetFromUrl,
}

#[derive(Clone, Debug)]
pub struct ClientConfig {
    pub url: String,
    pub method: HttpMethod,
    pub host: String,
    pub port: u16,
    pub user_agent: String,
    pub out_file: OutFile,
}

impl ClientConfig {
    /// Parse the configuration from the command line args
    pub fn parse<I>(args: I) -> Result<Self>
    where
        I: Iterator<Item = String>,
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
                (as $t:ty) => {{
                    let _next: $t = parse_next!();
                    _next
                }};
            }

            match arg.as_str() {
                "-m" | "--method" => conf.method = parse_next!(),
                "--host" => conf.host = parse_next!(),
                "-a" | "--user-agent" => conf.user_agent = parse_next!(),
                "-h" | "--help" => help(),
                "--license" => license(),
                "-O" => conf.out_file = OutFile::GetFromUrl,
                "-o" => conf.out_file = OutFile::Filename(parse_next!()),
                _ => conf.url = arg,
            }
        }

        let mut host = conf.url.as_str();
        host = conf.url.strip_prefix("http://").unwrap_or(host);

        let mut url = "/";

        if let Some(i) = host.find('/') {
            url = &host[i..];
            host = &host[..i];
        }

        if let Some(i) = host.find(':') {
            conf.port = host[i + 1..].parse()?;
            host = &host[..i];
        }

        conf.host = host.to_string();
        conf.url = url.to_string();

        if conf.host.is_empty() {
            return Err("Missing host".into());
        }

        Ok(conf)
    }
}

fn help() -> ! {
    println!(
        "\
http-client: Copyright (C) 2025 Saúl Valdelvira

This program is free software: you can redistribute it and/or modify it
under the terms of the GNU General Public License as published by the
Free Software Foundation, version 3.
Use http-client --license to read a copy of the GPL v3

USAGE: http-client [--method <HTTP Method>] [--host <hostname>]
                   [--user-agent <User Agent>] [-O] [-o <output-file>]
PARAMETERS:
    -m, --method    Set HTTP method
    --host  Set target hostname
    -a, --user-agent Set user agent string
    -O  Get output filename from the url
    -o  Set output filename
    -h, --help Display this help guide
    --license Display the license for this program"
    );
    process::exit(0);
}

fn license() -> ! {
    println!(include_str!("../COPYING"));
    process::exit(0);
}

trait ParseIterator {
    fn next_parse<T: FromStr>(&mut self) -> Option<T>;
}

impl<I> ParseIterator for I
where
    I: Iterator<Item = String>,
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
            url: String::new(),
            port: 80,
            host: String::new(),
            user_agent: "http-client".to_string(),
            out_file: OutFile::Stdout,
        }
    }
}

#[cfg(test)]
mod test {
    #![allow(clippy::unwrap_used)]

    use http::Result;

    use super::ClientConfig;

    fn parse_from_vec(v: &[&str]) -> Result<ClientConfig> {
        let conf = v.iter().map(|s| (*s).to_string());
        ClientConfig::parse(conf)
    }

    #[test]
    fn unknown() {
        let conf = vec!["?unknown"];
        let conf = parse_from_vec(&conf).unwrap();
        assert_eq!(conf.host, "?unknown");
    }
}
