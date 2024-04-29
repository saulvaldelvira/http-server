use std::{env::Args, str::FromStr};

pub struct Config {
    port: u16,
    n_threads: usize,
}

trait MyArgs {
    fn next_parse<T: FromStr>(&mut self) -> Option<T>;
}

impl MyArgs for Args {
    fn next_parse<T: FromStr>(&mut self) -> Option<T> {
        self.next()?.parse().ok()
    }
}

impl Config {
    pub fn parse(args: Args) -> Self {
        let mut conf = Self {
            port: 80,
            n_threads: 32,
        };
        let mut args = args.into_iter();
        while let Some(arg) = args.next() {
            match arg.as_str() {
                "-p" => conf.port = args.next_parse().expect("Missing port for argument -p"),
                "-n" => conf.n_threads = args.next_parse().expect("Missing number of threads for argument -n"),
                _ => {}
            }
        }
        conf
    }
    pub fn port(&self) -> u16 { self.port }
    pub fn n_threads(&self) -> usize { self.n_threads }
}
