use std::collections::HashMap;
use std::fs;
use std::fs::File;
use std::io::BufReader;
use std::io::ErrorKind::*;
use std::path::Path;

use crate::request::HttpRequest;
use crate::request::RequestMethod;
use crate::Result;

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
/// use http_server::request::handler::{self,*};
/// use http_server::request::RequestMethod;
///
/// let mut handler = Handler::new();
/// handler.get("/", |req| {
///     req.respond_buf(b"Hello world! :)")
/// });
/// handler.add_default(RequestMethod::GET, handler::cat_handler);
/// handler.post_interceptor(handler::log_request);
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
    pub fn get<F: HandlerFunc>(&mut self, url: &str, f: F) {
        self.add(RequestMethod::GET,url,f);
    }
    #[inline]
    pub fn post<F: HandlerFunc>(&mut self, url: &str, f: F) {
        self.add(RequestMethod::POST,url,f);
    }
    #[inline]
    pub fn delete<F: HandlerFunc>(&mut self, url: &str, f: F) {
        self.add(RequestMethod::DELETE,url,f);
    }
    /// Adds a handler for a request type
    ///
    /// - method: HTTP [method](RequestMethod) to match
    /// - url: URL for the handler
    /// - f: [Handler](HandlerFunc) for the request
    ///
    pub fn add<F: HandlerFunc>(&mut self, method: RequestMethod, url: &str, f: F) {
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
            Some(handler) => handler(req).or_else(|_| req.server_error()),
            None => req.forbidden(),
        };
        self.post_interceptors.iter().for_each(|f| f(req));
        ret
    }
}

fn head_headers(req: &mut HttpRequest) -> Result<()> {
    let filename = req.filename()?;
    match File::open(&filename) {
        Ok(file) => {
            let metadata = file.metadata()?;
            if metadata.is_file() {
                req.set_header("Content-Length", metadata.len());
            } else {
                req.set_status(404);
            }
        },
        Err(err) => {
            let status = match err.kind() {
                PermissionDenied => 403,
                _ => 404,
            };
            req.set_status(status);
        }
    };
    Ok(())
}

pub fn head_handler(req: &mut HttpRequest) -> Result<()> {
    head_headers(req)?;
    if req.status() != 200 {
        let page = req.error_page();
        req.set_header("Content-Length", page.len());
    }
    req.respond()
}

pub fn cat_handler(req: &mut HttpRequest) -> Result<()> {
    head_headers(req)?;
    if req.status() != 200 {
        return req.respond_error_page();
    };
    let file = File::open(req.filename()?)?;
    let mut reader = BufReader::new(file);
    req.respond_reader(&mut reader)
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

pub fn log_request(req: &mut HttpRequest) {
    println!("{} {} {} {}", req.method(), req.url(), req.status(), req.status_msg());
}

pub fn index_handler(req: &mut HttpRequest) -> Result<()> {
        if file_exists("index.html") {
            req.set_url("/index.html".to_owned());
            cat_handler(req)
        } else {
            req.respond_buf(
b"\
<!DOCTYPE html>
<html>
    <head>
        <title>HTTP Server</title>
    </head>
    <body>
        <h1>HTTP Server</h1>
        <p>Hello world :)</p>
    </body>
</html>")
        }
}
