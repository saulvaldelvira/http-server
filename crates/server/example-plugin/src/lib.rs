extern crate server;

use http::HttpRequest;
use server::{handler::Handler, http};

#[unsafe(no_mangle)]
pub extern "Rust" fn init_handler(handler: Option<&mut Handler>) {
    let handler = handler.unwrap();

    handler.get("/count", |req: &mut HttpRequest| {
        macro_rules! get_numeric_arg {
            ($n:literal) => {
                match req.param($n) {
                    Some(num) => match num.parse::<u32>() {
                        Ok(num) => num,
                        Err(err) => {
                            return req
                                .set_status(400)
                                .respond_str(&format!("Error parsing '{}': {err}", $n));
                        }
                    },
                    None => {
                        return req
                            .set_status(400)
                            .respond_str(concat!("Missing numeric argument ", $n));
                    }
                }
            };
        }
        let start = get_numeric_arg!("start");
        let end = get_numeric_arg!("end");

        req.respond_with(|out| {
            for i in start..=end {
                write!(out, "{i}, ")?;
            }
            Ok(())
        })
    });

    println!("example-plugin: Done");
}
