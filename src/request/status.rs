use super::HttpRequest;

impl HttpRequest {
    /// Get a human-readable description of the request's status code
    pub fn status_msg(&self) -> &'static str {
        match self.status {
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
