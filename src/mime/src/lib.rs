use core::str;
use std::{fmt::Display, path::Path, str::FromStr};

#[derive(Debug)]
pub struct Mime(String,String);

type Result<T> = std::result::Result<T,&'static str>;

impl Mime {
    pub fn from_str(text: &str) -> Result<Self> {
        let mut text = text.split("/");
        let major = text.next().ok_or("Malformatted mime type")?.to_owned();
        let minor = text.next().ok_or("Malformatted mime type")?.to_owned();
        Ok(Mime(major,minor))
    }
    pub fn from_filename(filename: &str) -> Result<Self> {
        let ext = Path::new(filename)
                       .extension().ok_or("Error getting file extension")?
                       .to_str().ok_or("Error convertion OsString to str")?;
        let major = match ext {
            "avi" => "video/x-msvideo",
            "aac" => "audio",
            "abw" => "application/x-abiword",
            "apng" | "avif" => "image",
            "arc" => "application/x-freearc",
            /* TODO: Complete */
            "json" => "application/json",
            "mp4" => "video",
            _ => return Err("Unknown extension")
        };
        Ok(
            if major.contains("/") {
                major.parse()?
            } else {
                Mime(major.to_owned(),ext.to_owned())
            }
          )
    }
    pub fn major(&self) -> &str { &self.0 }
    pub fn minor(&self) -> &str { &self.1 }
}

impl FromStr for Mime {
    type Err = &'static str;
    fn from_str(s: &str) -> Result<Self> {
        Mime::from_str(s)
    }
}

impl Display for Mime {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}/{}", self.major(),self.minor())
    }
}
