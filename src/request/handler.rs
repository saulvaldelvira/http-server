mod indexing;
mod ranges;
mod auth;
pub use auth::AuthConfig;

use std::collections::HashMap;
use std::fs;
use std::fs::File;
use std::fs::OpenOptions;
use std::io::stdout;
use std::io::BufReader;
use std::io::ErrorKind::*;
use std::io::Read;
use std::io::Seek;
use std::io::SeekFrom;
use std::io::Write;
use std::ops::DerefMut;
use std::ops::Range;
use std::path::Path;
use std::sync::Mutex;

use mime::Mime;

use crate::request::HttpRequest;
use crate::request::RequestMethod;
use crate::Result;
use self::indexing::index_of;
use self::ranges::get_range_for;

/// HandlerFunc trait
///
/// Represents a function that handles an [HttpRequest]
/// It receives a mutable reference to an [HttpRequest] and returns a [Result]<()>
pub trait HandlerFunc : Fn(&mut HttpRequest) -> Result<()> + Send + Sync + 'static { }
impl<T> HandlerFunc for T
where T: Fn(&mut HttpRequest) -> Result<()>  + Send + Sync + 'static { }

/// Interceptor trait
///
/// Represents a function that "intercepts" a request.
/// It can change it's state or log it's output.
pub trait Interceptor: Fn(&mut HttpRequest) + Send + Sync + 'static { }
impl<T> Interceptor for T
where T: Fn(&mut HttpRequest) + Send + Sync + 'static { }

pub type HandlerTable = HashMap<RequestMethod,HashMap<String,Box<dyn HandlerFunc>>>;

/// Handler
///
/// Matches [requests](HttpRequest) by their method and url, and
/// executes different handler functions for them.
///
/// # Example
/// ```
/// use http_srv::request::handler::{self,*};
/// use http_srv::request::RequestMethod;
///
/// let mut handler = Handler::new();
/// handler.get("/", |req| {
///     req.respond_buf(b"Hello world! :)")
/// });
/// handler.add_default(RequestMethod::GET, handler::cat_handler);
/// handler.post_interceptor(handler::log_stdout);
/// ```
pub struct Handler {
    handlers: HandlerTable,
    defaults: HashMap<RequestMethod,Box<dyn HandlerFunc>>,
    pre_interceptors: Vec<Box<dyn Interceptor>>,
    post_interceptors: Vec<Box<dyn Interceptor>>,
}

impl Handler {
    pub fn new() -> Self {
        Self { handlers: HashMap::new(), defaults: HashMap::new(),
               pre_interceptors: Vec::new(), post_interceptors: Vec::new() }
    }
    #[inline]
    pub fn get(&mut self, url: &str, f: impl HandlerFunc) {
        self.add(RequestMethod::GET,url,f);
    }
    #[inline]
    pub fn post(&mut self, url: &str, f: impl HandlerFunc) {
        self.add(RequestMethod::POST,url,f);
    }
    #[inline]
    pub fn delete(&mut self, url: &str, f: impl HandlerFunc) {
        self.add(RequestMethod::DELETE,url,f);
    }
    #[inline]
    pub fn head(&mut self, url: &str, f: impl HandlerFunc) {
        self.add(RequestMethod::HEAD,url,f);
    }
    /// Adds a handler for a request type
    ///
    /// - method: HTTP [method](RequestMethod) to match
    /// - url: URL for the handler
    /// - f: [Handler](HandlerFunc) for the request
    ///
    pub fn add(&mut self, method: RequestMethod, url: &str, f: impl HandlerFunc) {
        if !self.handlers.contains_key(&method) {
            self.handlers.insert(method, HashMap::new());
        }
        let map = self.handlers.get_mut(&method).unwrap();
        map.insert(url.to_string(), Box::new(f));
    }
    /// Adds a default handler for all requests of a certain type
    ///
    /// - method: HTTP [method](RequestMethod) to match
    /// - f: [Handler](HandlerFunc) for the requests
    ///
    #[inline]
    pub fn add_default(&mut self, method: RequestMethod, f: impl HandlerFunc) {
        self.defaults.insert(method, Box::new(f));
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
    pub fn get_handler(&self, method: &RequestMethod, url: &str) -> Option<&impl HandlerFunc> {
        match self.handlers.get(method) {
            Some(map) => map.get(url).or_else(|| self.defaults.get(method)),
            None => self.defaults.get(method),
        }
    }
    /// Handles a request if it finds a [HandlerFunc] for it.
    /// Else, it returns a 403 FORBIDDEN response
    pub fn handle(&self, req: &mut HttpRequest) -> Result<()> {
        self.pre_interceptors.iter().for_each(|f| f(req));
        let ret = match self.get_handler(req.method(), req.url()) {
            Some(handler) => handler(req).or_else(|err| {
                eprintln!("ERROR: {err}");
                req.server_error()
            }),
            None => req.forbidden(),
        };
        self.post_interceptors.iter().for_each(|f| f(req));
        ret
    }
}

impl Default for Handler {
    fn default() -> Self {
        let mut handler = Self::new();
        handler.pre_interceptor(suffix_html);
        handler.pre_interceptor(|req| {
            req.set_header("Accept-Ranges", "bytes");
        });

        handler.add_default(RequestMethod::GET, cat_handler);
        handler.add_default(RequestMethod::POST, post_handler);
        handler.add_default(RequestMethod::DELETE, delete_handler);
        handler.add_default(RequestMethod::HEAD, head_handler);

        handler.get("/", index_handler);
        handler.head("/", index_handler);

        handler.post_interceptor(log_stdout);
        handler
    }
}

fn head_headers(req: &mut HttpRequest) -> Result<Option<Range<u64>>> {
    let filename = req.filename()?;
    match File::open(&filename) {
        Ok(file) => {
            if let Ok(mime) = Mime::from_filename(&filename) {
                req.set_header("Content-Type", mime);
            }
            let metadata = file.metadata()?;
            let len = metadata.len();
            if metadata.is_file() {
                req.set_header("Content-Length", len);
            }
            let Some(range) = req.header("Range") else { return Ok(None); };
            let range = get_range_for(range, len)?;
            if range.end > len || range.end <= range.start {
                req.set_status(416);
            } else {
                req.set_status(206);
                req.set_header("Content-Length", range.end - range.start);
                req.set_header("Content-Range", &format!("bytes {}-{}/{}", range.start, range.end - 1, len));
            }
            return Ok(Some(range));
        },
        Err(err) => {
            let status = match err.kind() {
                PermissionDenied => 403,
                _ => 404,
            };
            req.set_status(status);
        }
    };
    Ok(None)
}

pub fn head_handler(req: &mut HttpRequest) -> Result<()> {
    head_headers(req)?;
    if !(200..300).contains(&req.status())  {
        let page = req.error_page();
        req.set_header("Content-Length", page.len());
    }
    let filename = req.filename()?;
    if dir_exists(&filename) {
        let page = index_of(&filename)?;
        req.set_header("Content-Length", page.len());
    }
    req.respond()
}

pub fn cat_handler(req: &mut HttpRequest) -> Result<()> {
    let range = head_headers(req)?;
    if !(200..300).contains(&req.status())  {
        return req.respond_error_page();
    };
    let filename = req.filename()?;
    if dir_exists(&filename) {
        let page = index_of(&filename)?;
        return req.respond_buf(page.as_bytes());
    }
    let mut file = File::open(req.filename()?)?;
    match range {
        Some(range) => {
            file.seek(SeekFrom::Start(range.start))?;
            let mut reader = BufReader::new(file)
                                       .take(range.end - range.start);
            req.respond_reader(&mut reader)
        },
        None => {
            let mut reader = BufReader::new(file);
            req.respond_reader(&mut reader)
        }
    }
}

pub fn post_handler(req: &mut HttpRequest) -> Result<()> {
    let filename = req.filename()?;
    match File::create(&filename) {
        Ok(mut file) => {
            req.read_data(&mut file)?;
            req.ok()
        },
        Err(err) => {
            println!("Error opening {}: {err}", &filename);
            match err.kind() {
                PermissionDenied => req.forbidden(),
                _ => req.not_found(),
            }
        }
    }
}

pub fn delete_handler(req: &mut HttpRequest) -> Result<()> {
    match fs::remove_file(req.filename()?) {
       Ok(_) => req.ok(),
       Err(err) =>
           match err.kind() {
                PermissionDenied => req.forbidden(),
                _ => req.not_found(),
           }
    }
}

fn file_exists(filename: &str) -> bool {
    Path::new(filename).is_file()
}

fn dir_exists(filename: &str) -> bool {
    Path::new(filename).is_dir()
}

pub fn suffix_html(req: &mut HttpRequest) {
    if file_exists(&req.url()[1..]) { return; }
    for suffix in [".html",".php"] {
        let mut filename = req.url().to_owned();
        filename.push_str(suffix);
        if file_exists(&filename[1..]) {
            req.set_url(filename);
            break;
        }
    }
}

#[inline]
fn log(w: &mut dyn Write, req: &HttpRequest) {
    write!(w, "{} {} {} {}\n", req.method(), req.url(), req.status(), req.status_msg()).unwrap();
}

pub fn log_stdout(req: &mut HttpRequest) {
    log(&mut stdout(), req);
}

pub fn log_file(filename: &str) -> impl Interceptor {
    let file = OpenOptions::new()
                .append(true)
                .create(true)
                .open(filename)
                .unwrap_or_else(|_| panic!("Error creating file: {filename}"));
    let file = Mutex::new(file);
    move |req| {
        let mut file = file.lock().unwrap();
        log(file.deref_mut(), req);
    }
}

pub fn index_handler(req: &mut HttpRequest) -> Result<()> {
        if file_exists("index.html") {
            req.set_url("/index.html".to_owned());
        }
        cat_handler(req)
}
