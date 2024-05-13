use std::fmt::Display;
use std::str::FromStr;

use crate::server::error::err;
use crate::server::ServerError;
use crate::Result;

/// Request Method
///
/// Represents the method of the HTTP request
#[derive(Debug,Eq,Hash,PartialEq,Clone,Copy)]
pub enum RequestMethod {
    GET, POST, PUT, DELETE,
    HEAD, PATCH, CONNECT,
    OPTIONS, TRACE,
}

impl FromStr for RequestMethod {
    type Err = ServerError;
    fn from_str(t: &str) -> Result<Self> {
        match t {
            "GET" => Ok(Self::GET),
            "POST" => Ok(Self::POST),
            "PUT" => Ok(Self::PUT),
            "DELETE" => Ok(Self::DELETE),
            "HEAD" => Ok(Self::HEAD),
            "PATCH" => Ok(Self::PATCH),
            "CONNECT" => Ok(Self::CONNECT),
            "OPTIONS" => Ok(Self::OPTIONS),
            "TRACE" => Ok(Self::TRACE),
            _ => err!("Couldn't parse request method ({t})")
        }
    }
}

impl Display for RequestMethod {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}
