use std::env;
use std::fs;
use std::io::stdin;
use std::io::stdout;
use std::io::Read;
use std::io::Write;
use std::process;

mod config;

use rb64::decode;
use rb64::encode;
use config::Config;

use crate::config::Operation;

fn main() -> std::io::Result<()> {
    let conf = Config::parse(env::args().skip(1)).unwrap();

    match conf.operation() {
        Operation::Encode => {
            for file in conf.files() {
                let data = fs::read(&file)?;
                let enc = encode(&data);
                if conf.files().len() > 1 {
                    let file = file.to_owned() + ".base64";
                    fs::write(&file, enc.as_bytes())?;
                } else {
                    stdout().write_all(enc.as_bytes())?;
                }

            }
            if conf.files().is_empty() {
                let mut data = Vec::new();
                stdin().read_to_end(&mut data)?;
                let enc = encode(&data);
                stdout().write_all(&enc.as_bytes())?;
            }
        },
        Operation::Decode => {
            for file in conf.files() {
                let data = fs::read_to_string(&file)?;
                let dec = decode(&data).unwrap_or_else(|err| {
                    println!("ERROR: {err}");
                    process::exit(1);
                });
                if conf.files().len() > 1 {
                    let file = file.to_owned() + ".decoded";
                    fs::write(&file, dec)?;
                } else {
                    stdout().write_all(&dec)?;
                }
            }
            if conf.files().is_empty() {
                let mut data = String::new();
                stdin().read_to_string(&mut data)?;
                let dec = decode(&data).unwrap_or_else(|err| {
                    println!("ERROR: {err}");
                    process::exit(1);
                });
                stdout().write_all(&dec)?;
            }
        }
    }
    Ok(())
}
