use std::collections::HashMap;
use std::fs;
use std::fs::File;
use std::io::BufReader;
use std::io::ErrorKind::*;

use crate::request::HttpRequest;
use crate::request::RequestMethod;
use crate::Result;

pub fn cat_handler(req: &mut HttpRequest) -> Result<()> {
    let filename = req.filename()?;
    match File::open(&filename) {
        Ok(file) => {
            let len = file.metadata()?.len();
            req.set_header("Content-Length", len);
            let mut reader = BufReader::new(file);
            req.respond_reader(&mut reader)
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
       Err(err) => {
           match err.kind() {
                PermissionDenied => req.forbidden(),
                _ => req.not_found(),
           }
       }
    }
}

/// HandlerFunc trait
///
/// Represents a function that handles an [HttpRequest]
/// It receives a mutable reference to an [HttpRequest] and returns a [Result]<()>
pub trait HandlerFunc : Fn(&mut HttpRequest) -> Result<()> + Send + Sync + 'static { }
impl<T> HandlerFunc for T
where T: Fn(&mut HttpRequest) -> Result<()>  + Send + Sync + 'static { }

pub type HandlerTable = HashMap<RequestMethod,HashMap<String,Box<dyn HandlerFunc>>>;

pub struct Handler {
    handlers: HandlerTable,
    defaults: HashMap<RequestMethod,Box<dyn HandlerFunc>>,
}

impl Handler {
    pub fn new() -> Self {
        Self { handlers: HashMap::new(), defaults: HashMap::new() }
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
    pub fn add_default<F: HandlerFunc>(&mut self, method: RequestMethod, f: F) {
        self.defaults.insert(method, Box::new(f));
    }
    /// Get the handler for a certain method and url
    pub fn get(&self, method: &RequestMethod, url: &str) -> Option<&Box<dyn HandlerFunc>> {
        match self.handlers.get(method) {
            Some(map) => map.get(url).or_else(|| self.defaults.get(method)),
            None => self.defaults.get(method),
        }
    }
    /// Handles a request if it finds a [HandlerFunc] for it.
    /// Else, it returns a 403 FORBIDDEN response
    pub fn handle(&self, req: &mut HttpRequest) -> Result<()> {
        match self.get(req.method(), req.url()) {
            Some(handler) => handler(req),
            None => req.forbidden(),
        }
    }
}
