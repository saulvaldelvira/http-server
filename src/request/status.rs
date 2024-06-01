use crate::HttpRequest;

pub trait StatusCode {
    /// Returns true if the status code of the
    /// request represents an OK status (200-300)
    fn is_http_ok(&self) -> bool;
    /// Returns true if the status code of the
    /// request doesn't represent an OK status (200-300)
    fn is_http_err(&self) -> bool;
    /// Get a human-readable description of the request's status code
    fn status_msg(&self) -> &'static str;
}

impl StatusCode for u16 {
    #[inline]
    fn is_http_ok(&self) -> bool {
        (200..300).contains(self)
    }
    #[inline]
    fn is_http_err(&self) -> bool {
        !self.is_http_ok()
    }
    fn status_msg(&self) -> &'static str {
        match self {
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
            _ => "?"
        }
    }
}

impl HttpRequest {
    #[inline]
    pub fn is_http_ok(&self) -> bool {
        self.status.is_http_ok()
    }
    #[inline]
    pub fn is_http_err(&self) -> bool {
        self.status.is_http_err()
    }
    #[inline]
    pub fn status_msg(&self) -> &'static str {
        self.status.status_msg()
    }
}
