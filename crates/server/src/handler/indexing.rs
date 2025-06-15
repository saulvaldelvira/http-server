use core::fmt::Write;
use std::{
    env,
    fs::{DirEntry, read_dir},
    path::Path,
};

use crate::Result;

macro_rules! path_to_str {
    ($path:expr) => {
        $path.to_str().ok_or_else(|| "Path is invalid Unicode")
    };
}

fn size_human(size: u64) -> String {
    const UNITS: [&str; 4] = ["bytes", "KiB", "MiB", "GiB"];
    let mut i = 0;
    let Ok(size) = u32::try_from(size) else {
        return format!("{size}");
    };
    let mut size = f64::from(size);
    while i < UNITS.len() {
        if size < 1024.0 {
            break;
        }
        size /= 1024.0;
        i += 1;
    }
    let size = (size * 10.0).round() / 10.0;
    let mut decimals = 0;
    if size.fract() > 0.0 && i == 3 {
        decimals = 1;
    }
    format!("{size:.decimals$} {}", UNITS[i])
}

fn encode_path(path: &Path, show_hidden: bool) -> Result<String> {
    let cap = path_to_str!(path)?.len();
    let mut encoded_url = String::with_capacity(cap);
    for part in path {
        let part = path_to_str!(part)?;
        encoded_url += "/";
        encoded_url += &url::encode(part)?;
    }
    if cap == 0 {
        encoded_url.push('/');
    }
    if !show_hidden {
        encoded_url += "?hidden=false";
    }
    Ok(encoded_url)
}

pub fn index_of(filename: &str, show_hidden: bool) -> Result<String> {
    let cwd_path = env::current_dir()?;
    let cwd = path_to_str!(&cwd_path)?;

    let mut html = String::from(
        "<html><head><meta charset=\"UTF-8\" />\
        <style>body{text-align:left;}\
        td{padding-right:1em;}\
        td:first-child{padding-right:0.2em;}</style></head><body>",
    );

    let title = Path::new(filename).strip_prefix(cwd)?;
    html.write_fmt(format_args!("<h1>Index of / {}</h1>", path_to_str!(title)?))?;

    let mut files = Vec::new();
    for f in read_dir(filename)? {
        files.push(f?);
    }
    files.sort_by_key(DirEntry::path);

    html.push_str("<table><tr><th>Name</th><th>Size</th></tr>");
    if let Some(parent) = Path::new(filename).parent() {
        if parent.starts_with(cwd) {
            let url = parent.strip_prefix(cwd)?;
            let url = encode_path(url, show_hidden)?;
            html.write_fmt(format_args!(
                "<tr><td>&larr;</td><td><a href=\"{url}\">..</a></td></tr>"
            ))?;
        }
    }
    for file in files {
        let path = file.path();
        let file = path.metadata()?;
        let text = path.strip_prefix(filename)?;
        let text = path_to_str!(text)?.to_owned();
        if !show_hidden && text.starts_with('.') {
            continue;
        }

        let icon = if file.is_dir() {
            "&#128447;"
        } else {
            "&#128456;"
        };
        let url = path.strip_prefix(cwd)?;

        let encoded_path = encode_path(url, show_hidden)?;
        html.write_fmt(format_args!(
            "<tr><td>{icon}</td><td><a href=\"{encoded_path}\">{text}</a></td>"
        ))?;
        html.write_fmt(format_args!("<td>{}</td>", size_human(file.len())))?;
        html.write_str("</tr>")?;
    }
    html.push_str("</table></body></html>");

    Ok(html)
}
