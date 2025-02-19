mod auth;
mod indexing;
mod ranges;
use std::{
    borrow::Cow,
    collections::HashMap,
    fs::{self, File, OpenOptions},
    io::{self, stdout, BufReader, Read, Seek, SeekFrom, Write},
    ops::Range,
    path::Path,
    sync::Mutex,
};

pub use auth::AuthConfig;
use http::HttpMethod;
use mime::Mime;
use regexpr::Regex;

use self::{indexing::index_of, ranges::get_range_for};
use crate::{request::HttpRequest, Result};

/* /// HandlerFunc trait */
/* /// */
/* /// Represents a function that handles an [HttpRequest] */
/* /// It receives a mutable reference to an [HttpRequest] and returns a [Result]<()> */
/* pub trait HandlerFunc : Fn(&mut HttpRequest) -> Result<()> + Send + Sync + 'static { } */
/* impl<T> HandlerFunc for T */
/* where T: */

/// Interceptor trait
///
/// Represents a function that "intercepts" a request.
/// It can change it's state or log it's output.
pub trait Interceptor: Fn(&mut HttpRequest) + Send + Sync + 'static {}
impl<T> Interceptor for T where T: Fn(&mut HttpRequest) + Send + Sync + 'static {}

#[derive(Default)]
struct HandlerMethodAssoc {
    exact: HashMap<String, Box<dyn RequestHandler>>,
    regex: Vec<(Regex, Box<dyn RequestHandler>)>,
    def: Option<Box<dyn RequestHandler>>,
}

pub trait RequestHandler: Send + Sync + 'static {
    /// Handler the request
    ///
    /// # Errors
    /// If some error ocurred while processing the request
    fn handle(&self, req: &mut HttpRequest) -> Result<()>;
}

impl<T> RequestHandler for T
where
    T: Fn(&mut HttpRequest) -> Result<()> + Send + Sync + 'static,
{
    fn handle(&self, req: &mut HttpRequest) -> Result<()> {
        self(req)
    }
}

enum UrlMatcherInner {
    Literal(String),
    Regex(Regex),
}

pub struct UrlMatcher(UrlMatcherInner);

impl UrlMatcher {
    pub fn regex(src: &str) -> Result<Self> {
        let regex = Regex::compile(src).map_err(|err| err.to_string())?;
        Ok(UrlMatcher(UrlMatcherInner::Regex(regex)))
    }
    #[must_use]
    pub fn literal(src: String) -> Self {
        UrlMatcher(UrlMatcherInner::Literal(src))
    }
}

impl From<&str> for UrlMatcher {
    fn from(value: &str) -> Self {
        Self::literal(value.to_string())
    }
}

impl From<String> for UrlMatcher {
    fn from(value: String) -> Self {
        Self::literal(value)
    }
}

/// Handler
///
/// Matches [requests](HttpRequest) by their method and url, and
/// executes different handler functions for them.
///
/// # Example
/// ```
/// use http_srv::handler::{self,*};
/// use http::request::HttpRequest;
/// use http::HttpMethod;
///
/// let mut handler = Handler::new();
/// handler.get("/", |req: &mut HttpRequest| {
///     req.respond_str("Hello world! :)")
/// });
/// handler.add_default(HttpMethod::GET, handler::cat_handler);
/// handler.post_interceptor(handler::log_stdout);
/// ```
pub struct Handler {
    handlers: HashMap<HttpMethod, HandlerMethodAssoc>,
    pre_interceptors: Vec<Box<dyn Interceptor>>,
    post_interceptors: Vec<Box<dyn Interceptor>>,
}

impl Handler {
    #[must_use]
    pub fn new() -> Self {
        Self {
            handlers: HashMap::new(),
            pre_interceptors: Vec::new(),
            post_interceptors: Vec::new(),
        }
    }
    /// Shortcut for [add](Handler::add)([`HttpMethod::GET`], ...)
    #[inline]
    pub fn get(&mut self, url: impl Into<UrlMatcher>, f: impl RequestHandler) {
        self.add(HttpMethod::GET, url, f);
    }
    /// Shortcut for [add](Handler::add)([`HttpMethod::POST`], ...)
    #[inline]
    pub fn post(&mut self, url: impl Into<UrlMatcher>, f: impl RequestHandler) {
        self.add(HttpMethod::POST, url, f);
    }
    /// Shortcut for [add](Handler::add)([`HttpMethod::DELETE`], ...)
    #[inline]
    pub fn delete(&mut self, url: impl Into<UrlMatcher>, f: impl RequestHandler) {
        self.add(HttpMethod::DELETE, url, f);
    }
    /// Shortcut for [add](Handler::add)([`HttpMethod::HEAD`], ...)
    #[inline]
    pub fn head(&mut self, url: impl Into<UrlMatcher>, f: impl RequestHandler) {
        self.add(HttpMethod::HEAD, url, f);
    }
    /// Adds a handler for a request type
    ///
    /// - method: HTTP [method](HttpMethod) to match
    /// - url: URL for the handler
    /// - f: [Handler](RequestHandler) for the request
    ///
    pub fn add(&mut self, method: HttpMethod, url: impl Into<UrlMatcher>, f: impl RequestHandler) {
        let map = self.handlers.entry(method).or_default();
        match url.into().0 {
            UrlMatcherInner::Literal(lit) => {
                map.exact.insert(lit, Box::new(f));
            }
            UrlMatcherInner::Regex(regex) => {
                map.regex.push((regex, Box::new(f)));
            }
        }
    }
    /// Adds a default handler for all requests of a certain type
    ///
    /// - method: HTTP [method](HttpMethod) to match
    /// - f: [Handler](RequestHandler) for the requests
    ///
    #[inline]
    pub fn add_default(&mut self, method: HttpMethod, f: impl RequestHandler) {
        self.handlers.entry(method).or_default().def = Some(Box::new(f));
    }
    /// Add a function to run before the request is processed
    #[inline]
    pub fn pre_interceptor(&mut self, f: impl Interceptor) {
        self.pre_interceptors.push(Box::new(f));
    }
    /// Add a function to run after the request is processed
    #[inline]
    pub fn post_interceptor(&mut self, f: impl Interceptor) {
        self.post_interceptors.push(Box::new(f));
    }
    /// Get the handler for a certain method and url
    #[must_use]
    pub fn get_handler(&self, method: &HttpMethod, url: &str) -> Option<&dyn RequestHandler> {
        let handler = self.handlers.get(method)?;

        handler
            .exact
            .get(url)
            .or_else(|| {
                handler
                    .regex
                    .iter()
                    .find(|(r, _)| r.test(url))
                    .map(|(_, f)| f)
            })
            .or(handler.def.as_ref())
            .map(|b| &**b)
    }
    /// Handles a request if it finds a [`RequestHandler`] for it.
    /// Else, it returns a 403 FORBIDDEN response
    pub fn handle(&self, req: &mut HttpRequest) -> Result<()> {
        self.pre_interceptors.iter().for_each(|f| f(req));
        let result = match self.get_handler(req.method(), req.url()) {
            Some(handler) => handler.handle(req).or_else(|err| {
                eprintln!("ERROR: {err}");
                req.server_error()
            }),
            None => req.forbidden(),
        };
        self.post_interceptors.iter().for_each(|f| f(req));
        result
    }
}

impl Default for Handler {
    /// Default Handler
    ///
    /// # Pre Interceptors
    ///  - [`suffix_html`]
    ///  - Set Header: "Accept-Ranges: bytes"
    ///
    /// # Handler Functions
    /// - [GET](HttpMethod::GET): [`cat_handler`]
    /// - [POST](HttpMethod::POST): [`post_handler`]
    /// - [DELETE](HttpMethod::DELETE): [`delete_handler`]
    /// - [HEAD](HttpMethod::HEAD): [`head_handler`]
    ///
    /// - [GET](HttpMethod::GET) "/": [`root_handler`]
    /// - [HEAD](HttpMethod::HEAD) "/": [`root_handler`]
    ///
    /// # Post Interceptors
    ///  - [`log_stdout`]
    ///
    fn default() -> Self {
        let mut handler = Self::new();
        handler.pre_interceptor(suffix_html);
        handler.pre_interceptor(|req| {
            req.set_header("Accept-Ranges", "bytes");
        });

        handler.add_default(HttpMethod::GET, cat_handler);
        handler.add_default(HttpMethod::POST, post_handler);
        handler.add_default(HttpMethod::DELETE, delete_handler);
        handler.add_default(HttpMethod::HEAD, head_handler);

        handler.get("/", root_handler);
        handler.head("/", root_handler);

        handler.post_interceptor(log_stdout);
        handler
    }
}

fn head_headers(req: &mut HttpRequest) -> Result<Option<Range<u64>>> {
    let filename = req.filename()?;
    if dir_exists(&filename) {
        req.set_header("Content-Type", "text/html");
        return Ok(None);
    }
    match File::open(&*filename) {
        Ok(file) => {
            if let Ok(mime) = Mime::from_filename(&filename) {
                req.set_header("Content-Type", mime.to_string());
            }
            let metadata = file.metadata()?;
            let len = metadata.len();
            if metadata.is_file() {
                req.set_header("Content-Length", len.to_string());
            }
            let Some(range) = req.header("Range") else {
                return Ok(None);
            };
            let range = get_range_for(range, len)?;
            if range.end > len || range.end <= range.start {
                req.set_status(416);
            } else {
                req.set_status(206);
                req.set_header("Content-Length", (range.end - range.start).to_string());
                req.set_header(
                    "Content-Range",
                    format!("bytes {}-{}/{}", range.start, range.end - 1, len),
                );
            }
            return Ok(Some(range));
        }
        Err(err) => {
            let status = match err.kind() {
                io::ErrorKind::PermissionDenied => 403,
                _ => 404,
            };
            req.set_status(status);
        }
    };
    Ok(None)
}

#[inline]
fn show_hidden(req: &HttpRequest) -> bool {
    match req.param("hidden") {
        Some(s) => s != "false",
        None => true,
    }
}

/// Returns the headers that would be sent by a [GET](HttpMethod::GET)
/// [request](HttpRequest), with an empty body.
pub fn head_handler(req: &mut HttpRequest) -> Result<()> {
    head_headers(req)?;
    let filename = req.filename()?;
    let len = if req.is_http_err() {
        req.error_page().len()
    } else if dir_exists(&filename) {
        index_of(&filename, show_hidden(req))?.len()
    } else {
        0
    };

    if len > 0 {
        req.set_header("Content-Length", len.to_string());
    }
    req.respond()
}

/// Returns the file, or an index of the directory.
///
/// # Errors
/// If the request returns an Error variant on send
pub fn cat_handler(req: &mut HttpRequest) -> Result<()> {
    let range = head_headers(req)?;
    if req.is_http_err() {
        return req.respond_error_page();
    };
    let filename = req.filename()?;
    if dir_exists(&filename) {
        let page = index_of(&filename, show_hidden(req))?;
        return req.respond_str(&page);
    }
    let mut file = File::open(&*req.filename()?)?;
    if let Some(range) = range {
        file.seek(SeekFrom::Start(range.start))?;
        let mut reader = BufReader::new(file).take(range.end - range.start);
        req.respond_reader(&mut reader)
    } else {
        let mut reader = BufReader::new(file);
        req.respond_reader(&mut reader)
    }
}

/// Save the data of the request to the url
///
/// # Errors
/// If the request returns an Error variant on send
pub fn post_handler(req: &mut HttpRequest) -> Result<()> {
    let filename = req.filename()?;
    match File::create(&*filename) {
        Ok(mut file) => {
            req.read_body(&mut file)?;
            req.ok()
        }
        Err(err) => {
            println!("Error opening {}: {err}", &filename);
            match err.kind() {
                io::ErrorKind::PermissionDenied => req.forbidden(),
                _ => req.not_found(),
            }
        }
    }
}

/// Delete the filename
///
/// # Errors
/// If the request returns an Error variant on send
pub fn delete_handler(req: &mut HttpRequest) -> Result<()> {
    match fs::remove_file(&*req.filename()?) {
        Ok(()) => req.ok(),
        Err(err) => match err.kind() {
            io::ErrorKind::PermissionDenied => req.forbidden(),
            _ => req.not_found(),
        },
    }
}

#[inline]
fn file_exists(filename: &str) -> bool {
    Path::new(filename).is_file()
}

#[inline]
fn dir_exists(filename: &str) -> bool {
    Path::new(filename).is_dir()
}

/// Appends a suffix to the url
///
/// If the requested url doesn't exists, try to
/// append a suffix ('.html', '.php'), and if it
/// exists, modify the url.
pub fn suffix_html(req: &mut HttpRequest) {
    if file_exists(&req.url()[1..]) {
        return;
    }
    for suffix in [".html", ".php"] {
        let mut filename = req.url().to_owned();
        filename.push_str(suffix);
        if file_exists(&filename[1..]) {
            req.set_url(filename);
            break;
        }
    }
}

macro_rules! log {
    ($w:expr , $req:expr) => {
        writeln!(
            $w,
            "{} {} {} {}",
            $req.method(),
            $req.url(),
            $req.status(),
            $req.status_msg()
        )
        .unwrap();
    };
}

/// Log the [request](HttpRequest) to stdout
pub fn log_stdout(req: &mut HttpRequest) {
    log!(&mut stdout(), req);
}

/// Log the [request](HttpRequest) to a file
///
/// The file is appended to, or created if it doesn't exists
///
/// # Errors
/// If creating the log file fails
#[allow(clippy::missing_panics_doc)]
pub fn log_file(filename: &str) -> crate::Result<Box<dyn Interceptor>> {
    let file = OpenOptions::new()
        .append(true)
        .create(true)
        .open(filename)
        .map_err(|err| Cow::Owned(format!("Error creating file: {filename}: {err}")))?;
    let file = Mutex::new(file);
    Ok(Box::new(move |req: &mut HttpRequest| {
        #[allow(clippy::unwrap_used)]
        let mut file = file.lock().unwrap();
        log!(&mut *file, req);
    }))
}

/// Rewrites / to /index.html
///
/// # Errors
/// If the request returns an Error variant on send
pub fn root_handler(req: &mut HttpRequest) -> Result<()> {
    if file_exists("index.html") {
        req.set_url("/index.html");
    }
    cat_handler(req)
}

pub fn redirect(uri: impl Into<Box<str>>) -> impl RequestHandler {
    let uri = uri.into();
    move |req: &mut HttpRequest| {
        req.set_header("Location", &*uri);
        req.set_header("Content-Length", "0");
        req.set_status(308).respond()
    }
}
