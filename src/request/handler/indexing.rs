use std::{env, fs::read_dir, path::Path};

use crate::Result;
use html::*;

macro_rules! path_to_str {
    ($path:expr) => {
        $path.to_str().ok_or_else(|| "Path is invalid Unicode")
    };
}

pub fn index_of(filename: &str) -> Result<String> {
    let cwd_path = env::current_dir()?;
    let cwd = path_to_str!(&cwd_path)?;

    let title = Path::new(filename).strip_prefix(cwd)?;
    let title = "Index of /".to_owned() + path_to_str!(title)?;
    let mut html = HtmlBuilder::with_title(&title);
    html.body()
        .append(html!("h1",{text: &title}));

    let mut pairs = Vec::new();
    for file in read_dir(filename)? {
        let path = file?.path();

        let text = path.strip_prefix(filename)?;
        let icon =
            if path.is_dir() {
                "&#128447; "
            } else {
                "&#128457; "
            }.to_owned();
        let text = icon + path_to_str!(text)?;

        let url = path.strip_prefix(&cwd)?;
        let mut encoded_url = "".to_owned();
        for part in url {
            let part = path_to_str!(part)?;
            encoded_url += "/";
            encoded_url += &url::encode(part)?;
        }
        pairs.push((text,encoded_url));
    }
    pairs.sort_by(|a,b| a.0.cmp(&b.0));

    let lis = pairs
        .iter()
        .map(|p|
                html!("li",[
                            html!("a",
                               {"href": &p.1},
                               {text: &p.0})
                            ])
            );
    html.body()
        .append(
           html!("ul", {append_iter: lis}));
    Ok(html.to_string())
}

