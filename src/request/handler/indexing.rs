use std::{env, fs::read_dir};

use crate::{server::ServerError, Result};
use html::*;

macro_rules! path_to_str {
    ($path:expr) => {
        $path.to_str().ok_or_else(|| ServerError::from_str("Path is invalid Unicode"))
    };
}

pub fn index_of(filename: &str) -> Result<String> {
    let title = "Index of ".to_owned() + filename;
    let mut html = HtmlBuilder::with_title(&title);
    html.body()
        .append(html!("h1",{text: &title}));

    let cwd_path = env::current_dir()?;
    let cwd = path_to_str!(&cwd_path)?;

    let mut pairs = Vec::new();
    for file in read_dir(filename)? {
        let path = file?.path();

        let text = path.strip_prefix(filename)?;
        let text = path_to_str!(text)?.to_owned();

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

