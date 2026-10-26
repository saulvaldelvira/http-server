use std::ffi::OsStr;
use std::io::{BufRead, BufReader};
use std::os::unix::ffi::OsStrExt;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use http::HttpError;
use core::fmt::Write;

use super::{HttpRequest, Result};

pub fn cgi_handler(req: &mut HttpRequest) -> Result<()> {
    let filename = req.filename()?;

    let mut script_path = PathBuf::new();
    let mut path2 = PathBuf::new();

    let mut segments = Path::new(OsStr::from_bytes(filename.as_bytes())).iter();
    while let Some(segment) = segments.next() {
        script_path.push(segment);
        if Path::new(segment).extension().is_some_and(|e| e == "cgi") {
            path2.push("/");
            for segment in segments.by_ref() {
                path2.push(segment);
            }
        }
    }

    if !script_path.exists() {
        return req.set_status(404).respond_error_page();
    }

    let mut query = String::new();
    // TODO: Maybe store the raw query?
    for (k, v) in req.params() {
        if !query.is_empty() { query.push('&'); }
        query.write_fmt(format_args!("{k}={v}"))?;
    }
    let query = url::encode(&query)?;

    let output = Command::new(script_path)
        .stdout(Stdio::piped())
        .env("PATH_INFO", path2)
        .env("QUERY_STRING", query.into_owned())
        // TODO: More env variables
        .spawn()?
        .wait_with_output()?;

    let mut br = BufReader::new(output.stdout.as_slice());
    for line in
    (&mut br).lines().map_while(|l| {
        let l = l.ok()?;
        (!l.is_empty()).then_some(l)
    })
    {
        let (k, v) = line.split_once(": ").ok_or_else(|| HttpError::new("Expected a header list"))?;
        req.set_header(k, v);
    }

    if output.status.success() {
        req.respond_reader(&mut br)
    } else {
        req.server_error()
    }
}
