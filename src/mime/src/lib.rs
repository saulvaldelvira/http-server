//! Mime Type crate
//!
//! This crate contains utils to work with MIME types.
//!
//! # Example
//! ```
//! use rmime::Mime;
//!
//! let mime = Mime::from_filename("my_video.mp4").unwrap();
//! assert_eq!(mime.to_string(), "video/mp4");
//! ```

use core::str;
use std::{borrow::Cow, fmt::Display, path::Path};

/// Mime Type struct
///
/// This struct represents a Mime type.
/// It contains a major and a minor type.
#[derive(Debug)]
pub struct Mime<'a>(Cow<'a,str>,Cow<'a,str>);

type Result<T> = std::result::Result<T,&'static str>;

impl<'a> Mime<'a> {
    /// Create a MIME type from a given string.
    ///
    /// It can either be a reference or an owned String
    ///
    /// If the MIME type is built from a reference, it
    /// can't outlive the reference.
    ///
    /// # Example
    /// This piece of code fails to compile, since the mime
    /// struct is used after the String it references is gone.
    ///
    /// ```compile_fail
    /// use rmime::Mime;
    ///
    /// let mime: Mime;
    /// {
    ///     let string = "text/plain".to_string();
    ///     mime = Mime::new(&string).unwrap();
    /// }
    /// mime.to_string();
    /// ```
    ///
    /// You can fix this by using the [into_owned](Self::into_owned) method.
    ///
    /// ```
    /// use rmime::Mime;
    ///
    /// let mime: Mime;
    /// {
    ///     let string = "text/plain".to_string();
    ///     mime = Mime::new(&string).unwrap().into_owned();
    /// }
    /// mime.to_string();
    /// ```
    pub fn new(text: impl Into<Cow<'a,str>>) -> Result<Self> {
        match text.into() {
            Cow::Owned(own) => {
                let mut text = own.split('/');
                let major = text.next().ok_or("Malformatted mime type")?.to_owned();
                let minor = text.next().ok_or("Malformatted mime type")?.to_owned();
                Ok(Mime(major.to_owned().into(),minor.to_owned().into()))
            },
            Cow::Borrowed(borr) => {
                let mut text = borr.split('/');
                let major = text.next().ok_or("Malformatted mime type")?;
                let minor = text.next().ok_or("Malformatted mime type")?;
                Ok(Mime(major.into(),minor.into()))
            }
        }
    }
    /// Creates a MIME type from the given filename.
    ///
    /// # Example
    /// ```
    /// use rmime::Mime;
    ///
    /// let mime = Mime::from_filename("my_video.mp4").unwrap();
    /// assert_eq!(mime.to_string(), "video/mp4");
    ///
    /// let mime = Mime::from_filename("index.html").unwrap();
    /// assert_eq!(mime.to_string(), "text/html");
    /// ```
    pub fn from_filename(filename: &'a str) -> Result<Self> {
        let ext =
            match Path::new(filename).extension() {
                Some(ext) => ext.to_str()
                                .ok_or("Error convertion OsString to str")?,
                None => ""
            };
        let major = match ext {
            "abw" => "application/x-abiword",
            "arc" => "application/x-freearc",
            "avi" => "video/x-msvideo",
            "azw" => "application/vnd.amazon.ebook",
            "bin" => "application/octet-stream",
            "bz" => "application/x-bzip",
            "bz2" => "application/x-bzip2",
            "cda" => "application/x-cdf",
            "csh" => "application/x-csh",
            "doc" => "application/msword",
            "docx" => "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
            "eot" => "application/vnd.ms-fontobject",
            "epub" => "application/epub+zip",
            "gz" => "application/gzip",
            "ico" => "image/vnd.microsoft.icon",
            "ics" => "text/calendar",
            "jar" => "application/java-archive",
            "js" => "text/javascript",
            "jsonld" => "application/ld+json",
            "mid" | "midi" => "audio/x-midi",
            "mjs" => "text/javascript",
            "mp3" => "audio/mpeg",
            "mpkg" => "application/vnd.apple.installer+xml",
            "odp" => "application/vnd.oasis.opendocument.presentation",
            "ods" => "application/vnd.oasis.opendocument.spreadsheet",
            "odt" => "application/vnd.oasis.opendocument.text",
            "oga" | "ogv" => "audio/ogg",
            "ogx" => "application/ogg",
            "php" => "application/x-httpd-php",
            "ppt" => "application/vnd.ms-powerpoint",
            "pptx" => "application/vnd.openxmlformats-officedocument.presentationml.presentation",
            "rar" => "application/vnd.rar",
            "sh" => "application/x-sh",
            "sgv" => "image/svg+xml",
            "tar" => "application/x-tar",
            "tif" | "tiff" => "image/tiff",
            "ts" => "video/mp2t",
            "vsd" => "application/vnd.visio",
            "weba" => "audio/webm",
            "xhtml" => "application/xhtml+xml",
            "xls" => "application/vnd.ms-excel",
            "xlsx" => "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
            "xul" => "application/vnd.mozilla.xul+xml",
            "3gp" => "video/3gpp",
            "3g2" => "video/3gpp2",
            "7z" => "application/x-7z-compressed",
            "apng" | "avif" | "bmp" | "gif" |
            "jpeg" | "png"  | "webp"           => "image",
            "jpg" => "image/jpeg",
            "html" | "htm"  | "css"  | "csv"   => "text",
            "otf"  | "ttf"  | "woff" | "woff2" => "font",
            "json" | "rtf"  | "xml"  | "zip"   => "application",
            "aac"  | "opus" | "wav"  | "flac"  => "audio",
            "mp4"  | "mpeg" | "webm" | "mkv"   => "video",
            "" | "txt" => "text/plain",
            _ => return Err("Unknown extension")
        };
        Ok(
            if major.contains('/') {
                Mime::new(major)?
            } else {
                Mime(major.into(),ext.into())
            }
          )
    }
    pub fn into_owned(self) -> Mime<'static> {
        Mime(self.0.into_owned().into(),
             self.1.into_owned().into())
    }
    pub fn major(&self) -> &str { &self.0 }
    pub fn minor(&self) -> &str { &self.1 }
}

impl Display for Mime<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}/{}", self.major(),self.minor())
    }
}

