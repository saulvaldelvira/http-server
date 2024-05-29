use std::{env, fs::read_dir, path::Path};

use crate::Result;
use html::*;

macro_rules! path_to_str {
    ($path:expr) => {
        $path.to_str().ok_or_else(|| "Path is invalid Unicode")
    };
}

fn size_human(size: u64) -> String {
    const UNITS: [&str; 4] = ["bytes", "KiB", "MiB", "GiB"];
    let mut i = 0;
    let mut size = size as f64;
    while i < UNITS.len() {
        if size < 1024.0 { break; }
        size /= 1024.0;
        i += 1;
    }
    let size = (size * 10.0).round() / 10.0;
    let decimals = if size.fract() > 0.0 { 1 } else { 0 };
    format!("{size:.decimals$} {}", UNITS[i])
}

pub fn index_of(filename: &str) -> Result<String> {
    let cwd_path = env::current_dir()?;
    let cwd = path_to_str!(&cwd_path)?;

    let title = Path::new(filename).strip_prefix(cwd)?;
    let title = "Index of /".to_owned() + path_to_str!(title)?;
    let mut html = HtmlBuilder::with_title(&title);
    html.body()
        .append(html!("h1",{text: &title}));
    html.head().append(html!("style", {text: "
        body {
            text-align: left;
        }
        td {
            padding-right: 1em;
        }
        td:first-child {
            padding-right: 0.2em;
        }
        "}));

    let mut files = Vec::new();
    for f in read_dir(filename)? {
        files.push(f?);
    }
    files.sort_by(|a,b| a.path().cmp(&b.path()));

    let mut table = html!("table", [
        html!("tr", [
            html!("th"),
            html!("th", {text: "Name"}),
            html!("th", {text: "Size"}),
        ])
    ]);
    for file in files {
        let path = file.path();
        let file = path.metadata()?;
        let icon =
            if file.is_dir() {
                "&#128447;"
            } else {
                "&#128456;"
            };
        let text = path.strip_prefix(filename)?;
        let text = path_to_str!(text)?;

        let url = path.strip_prefix(&cwd)?;
        let mut encoded_url = "".to_owned();
        for part in url {
            let part = path_to_str!(part)?;
            encoded_url += "/";
            encoded_url += &url::encode(part)?;
        }
        let mut tr = html!("tr", [
            html!("td", {text: icon}),
            html!("td", [
                html!("a", {"href": encoded_url}, {text: text}),
            ]),
        ]);
        if file.is_file() {
            html!("td", {text: size_human(file.len())}).append_to(&mut tr);
        }
        table.append(tr);
    }

    html.body().append(table);
    Ok(html.to_string())
}

