pub trait StatusCode {
    /// Returns true if the status code of the
    /// request represents an OK status (200-300)
    fn is_http_ok(&self) -> bool;
    /// Returns true if the status code of the
    /// request doesn't represent an OK status (200-300)
    fn is_http_err(&self) -> bool;

    /// Returns true if the status code
    /// indicates am user error (4XX)
    fn is_user_err(&self) -> bool;
    /// Returns true if the status code
    /// indicates a server error (5XX)
    fn is_server_err(&self) -> bool;
    /// Get a human-readable description of the request's status code
    fn status_msg(&self) -> &'static str;
}

macro_rules! into {
    ($e:expr) => {
        <Self as TryInto<u64>>::try_into(*$e)
    };
}

impl<T: TryInto<u64> + Copy> StatusCode for T {
    #[inline]
    fn is_http_ok(&self) -> bool {
        into!(self).is_ok_and(|n| (200..300).contains(&n))
    }
    #[inline]
    fn is_user_err(&self) -> bool {
        into!(self).is_ok_and(|n| (400..500).contains(&n))
    }
    #[inline]
    fn is_server_err(&self) -> bool {
        into!(self).is_ok_and(|n| (500..600).contains(&n))
    }
    #[inline]
    fn is_http_err(&self) -> bool {
        !self.is_http_ok()
    }

    fn status_msg(&self) -> &'static str {
        let Ok(n) = into!(self) else {
            return "?";
        };
        match n {
            200 => "OK",
            201 => "CREATED",
            202 => "ACCEPTED",
            203 => "NON-AUTHORITATIVE INFORMATION",
            204 => "NO CONTENT",
            205 => "RESET CONTENT",
            206 => "PARTIAL CONTENT",
            300 => "MULTIPLE CHOICES",
            301 => "MOVED PERMANENTLY",
            302 => "FOUND",
            303 => "SEE OTHER",
            304 => "NOT MODIFIED",
            307 => "TEMPORARY REDIRECT",
            308 => "PERMANENT REDIRECT",
            400 => "BAD REQUEST",
            401 => "UNAUTHORIZED",
            403 => "FORBIDDEN",
            404 => "NOT FOUND",
            405 => "METHOD NOT ALLOWED",
            406 => "NOT ACCEPTABLE",
            407 => "PROXY AUTHENTICATION REQUIRED",
            408 => "REQUEST TIMEOUT",
            409 => "CONFLICT",
            410 => "GONE",
            411 => "LENGTH REQUIRED",
            412 => "PRECONDITION FAILED",
            413 => "PAYLOAD TOO LARGE",
            414 => "URI TOO LONG",
            415 => "UNSUPPORTED MEDIA TYPE",
            416 => "REQUESTED RANGE NOT SATISFIABLE",
            429 => "TOO MANY REQUESTS",
            501 => "NOT IMPLEMENTED",
            500 => "INTERNAL SERVER ERROR",
            _ => "?",
        }
    }
}

#[cfg(test)]
mod test {
    use super::StatusCode;

    #[test]
    fn code_test() {
        for n in 200..300 {
            assert!(n.is_http_ok());
            assert!(!n.is_http_err());
        }
        assert!(!300.is_http_ok());
        assert!(510.is_server_err());
        assert!(!600.is_server_err());
    }
}
