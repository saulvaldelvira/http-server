use std::collections::HashMap;
use std::fs::File;
use std::io::BufRead;
use std::io::BufReader;
use std::ops::Add;

use crate::Result;
use crate::HttpRequest;
use super::HandlerFunc;
use crate::server::error::err;

/// Authentication Config
///
/// Helps wrapping [handler functions](HandlerFunc) behing authentication
///
/// # Example
/// ```
/// use http_srv::prelude::*;
/// use http_srv::request::handler::AuthConfig;
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
    users: HashMap<String,String>,
    required_users: Vec<String>,
}

impl AuthConfig {
    pub fn of_file(filename: &str) -> Self {
        let f = File::open(filename).unwrap();
        let f = BufReader::new(f);
        let mut users = HashMap::new();
        let mut lines = f.lines();
        while let Some(Ok(l))  = lines.next() {
            let mut l = l.split_whitespace();
            let u = l.next().unwrap().to_owned();
            let p = l.next().unwrap().to_owned();
            users.insert(u,p);
        }
        users.shrink_to_fit();
        Self {
            users,
            required_users: Vec::new(),
        }
    }
    pub fn of_list(list: & [(& str,& str)]) -> Self {
        let mut users = HashMap::new();
        list.iter().for_each(|e| { users.insert(e.0.to_owned(), e.1.to_owned()); } );
        Self {
            users,
            required_users: Vec::new(),
        }
    }
    pub fn require_user(mut self, user: &str) -> Self {
        self.required_users.push(user.to_owned());
        self
    }
    pub fn apply(&self, f: impl HandlerFunc) -> impl HandlerFunc {
        let users = self.required_users.clone();
        let passwd = self.users.clone();
        move |req: &mut HttpRequest| {
            let Some(auth) = req.header("Authorization") else {
                req.set_header("WWW-Authenticate", "Basic");
                return req.unauthorized();
            };
            let auth = HttpAuth::parse(auth)?;
            if auth.check(&users, &passwd) {
                f(req)
            } else {
                req.unauthorized()
            }
        }
    }
}

impl<F> Add<F> for &AuthConfig
where
    F: HandlerFunc
{
    type Output = Box<dyn HandlerFunc>;

    fn add(self, rhs: F) -> Self::Output {
        Box::new(self.apply(rhs))
    }
}


#[derive(Clone, PartialEq, Debug)]
enum HttpAuth {
    Basic(String,String),
}

impl HttpAuth {
    fn parse(header: &str) -> Result<Self> {
        let mut auth = header.split_whitespace();
        let t = auth.next().ok_or("Malfromatted Authentication header")?;
        let payload = auth.next().ok_or("Malfromatted Authentication header")?;

        match t {
            "Basic" => parse_basic(payload),
            _ => err!("Unknown authentication method {t}")
        }
    }
    fn check(&self, users: &[String], passwds: &HashMap<String,String>) -> bool {
        match self {
            HttpAuth::Basic(user,pass) => {
                if users.is_empty() || users.contains(user) {
                    if let Some(p) = passwds.get(user).as_ref() {
                        *p == pass
                    } else {
                        false
                    }
                } else {
                    false
                }
            },
        }
    }
}

fn parse_basic(payload: &str) -> Result<HttpAuth> {
    let decoded = base64::decode(payload)?;
    let decoded = String::from_utf8(decoded)?;
    let mut decoded = decoded.splitn(2, ':');
    let user = decoded.next().unwrap_or("");
    let passwd = decoded.next().unwrap_or("");
    let user = url::decode(user)?.into_owned();
    let passwd = url::decode(passwd)?.into_owned();
    Ok(HttpAuth::Basic(user,passwd))
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test() {
        let auth = HttpAuth::parse("Basic dXNlcjpwYXNzd2Q=").expect("Expected correct parsing");
        match auth {
            HttpAuth::Basic(user,passwd) => {
                assert_eq!(user, "user");
                assert_eq!(passwd, "passwd");
            },
        }
    }
}
