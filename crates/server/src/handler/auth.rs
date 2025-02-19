use core::str;
use std::{
    collections::HashMap,
    fs::File,
    io::{BufRead, BufReader},
    sync::Arc,
};

use super::RequestHandler;
use crate::{err, HttpRequest, Result};

/// Authentication Config
///
/// Helps wrapping a [request handler](RequestHandler) behing authentication
///
/// # Example
/// ```
/// use http::*;
/// use http_srv::handler::*;
///
/// let auth = AuthConfig::of_list(&[("user", "passwd")]);
///
/// let mut handler = Handler::default();
/// let func = |req: &mut HttpRequest| {
///     req.respond_str("Super secret message")
/// };
/// handler.get("/secret", auth.apply(func));
/// ```
pub struct AuthConfig {
    users: Arc<HashMap<String, String>>,
    required_users: Arc<Vec<String>>,
}

pub struct AuthConfigBuilder {
    users: HashMap<String, String>,
    required_users: Vec<String>,
}

impl AuthConfigBuilder {
    pub fn require_user(mut self, user: &str) -> Self {
        self.required_users.push(user.to_owned());
        self
    }
    pub fn build(self) -> AuthConfig {
        AuthConfig {
            users: Arc::new(self.users),
            required_users: Arc::new(self.required_users),
        }
    }
}

impl AuthConfig {
    #[must_use]
    pub fn builder() -> AuthConfigBuilder {
        AuthConfigBuilder {
            users: HashMap::new(),
            required_users: Vec::new(),
        }
    }
    pub fn of_file(filename: &str) -> crate::Result<Self> {
        let f = File::open(filename)?;
        let f = BufReader::new(f);
        let mut users = HashMap::new();
        let mut lines = f.lines();
        while let Some(Ok(l)) = lines.next() {
            let mut l = l.split_whitespace();
            let u = l.next().ok_or("Malformatted file")?.to_owned();
            let p = l.next().ok_or("Malformatted file")?.to_owned();
            users.insert(u, p);
        }
        users.shrink_to_fit();
        Ok(Self {
            users: Arc::new(users),
            required_users: Arc::new(Vec::new()),
        })
    }
    #[must_use]
    pub fn of_list(list: &[(&str, &str)]) -> Self {
        let mut users = HashMap::new();
        for e in list {
            users.insert(e.0.to_owned(), e.1.to_owned());
        }
        Self {
            users: Arc::new(users),
            required_users: Arc::new(Vec::new()),
        }
    }
    pub fn apply<H: RequestHandler>(&self, f: H) -> AuthedRequest<H> {
        AuthedRequest {
            f,
            users: Arc::clone(&self.users),
            required_users: Arc::clone(&self.required_users),
        }
    }
}

pub struct AuthedRequest<H: RequestHandler> {
    f: H,
    users: Arc<HashMap<String, String>>,
    required_users: Arc<Vec<String>>,
}

impl<H: RequestHandler> RequestHandler for AuthedRequest<H> {
    fn handle(&self, req: &mut HttpRequest) -> Result<()> {
        let Some(auth) = req.header("Authorization") else {
            req.set_header("WWW-Authenticate", "Basic");
            return req.unauthorized();
        };
        let auth = HttpAuth::parse(auth)?;
        if auth.check(&self.required_users, &self.users) {
            self.f.handle(req)
        } else {
            req.unauthorized()
        }
    }
}

#[derive(Clone, PartialEq, Debug)]
enum HttpAuth {
    Basic(String, String),
}

impl HttpAuth {
    fn parse(header: &str) -> Result<Self> {
        let mut auth = header.split_whitespace();
        let t = auth.next().ok_or("Malfromatted Authentication header")?;
        let payload = auth.next().ok_or("Malfromatted Authentication header")?;

        match t {
            "Basic" => parse_basic(payload),
            _ => err!("Unknown authentication method {t}"),
        }
    }
    fn check(&self, users: &[String], passwds: &HashMap<String, String>) -> bool {
        match self {
            HttpAuth::Basic(user, pass) => {
                if users.is_empty() || users.contains(user) {
                    if let Some(p) = passwds.get(user).as_ref() {
                        *p == pass
                    } else {
                        false
                    }
                } else {
                    false
                }
            }
        }
    }
}

fn parse_basic(payload: &str) -> Result<HttpAuth> {
    let decoded = base64::decode(payload)?;
    let decoded = str::from_utf8(&decoded)?;
    let mut decoded = decoded.splitn(2, ':');
    let user = decoded.next().unwrap_or("");
    let passwd = decoded.next().unwrap_or("");
    let user = url::decode(user)?.into_owned();
    let passwd = url::decode(passwd)?.into_owned();
    Ok(HttpAuth::Basic(user, passwd))
}

#[cfg(test)]
mod test {
    #![allow(clippy::expect_used)]
    use super::*;

    #[test]
    fn test() {
        let auth = HttpAuth::parse("Basic dXNlcjpwYXNzd2Q=").expect("Expected correct parsing");
        match auth {
            HttpAuth::Basic(user, passwd) => {
                assert_eq!(user, "user");
                assert_eq!(passwd, "passwd");
            }
        }
    }
}
