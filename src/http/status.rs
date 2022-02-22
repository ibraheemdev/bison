use std::fmt;

/// Status of an HTTP response.
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub enum Status {
    /// 100 Continue
    Continue,

    /// 101 Switching Protocols
    SwitchingProtocols,

    /// 103 Early Hints
    EarlyHints,

    /// 200 Ok
    Ok,

    /// 201 Created
    Created,

    /// 202 Accepted
    Accepted,

    /// 203 Non Authoritative Information
    NonAuthoritativeInformation,

    /// 204 No Content
    NoContent,

    /// 205 Reset Content
    ResetContent,

    /// 206 Partial Content
    PartialContent,

    /// 207 Multi-Status
    MultiStatus,

    /// 226 Im Used
    ImUsed,

    /// 300 Multiple Choice
    MultipleChoice,

    /// 301 Moved Permanently
    MovedPermanently,

    /// 302 Found
    Found,

    /// 303 See Other
    SeeOther,

    /// 304 Not Modified
    NotModified,

    /// 307 Temporary Redirect
    TemporaryRedirect,

    /// 308 Permanent Redirect
    PermanentRedirect,

    /// 400 Bad Request
    BadRequest,

    /// 401 Unauthorized
    Unauthorized,

    /// 402 Payment Required
    PaymentRequired,

    /// 403 Forbidden
    Forbidden,

    /// 404 Not Found
    NotFound,

    /// 405 Method Not Allowed
    MethodNotAllowed,

    /// 406 Not Acceptable
    NotAcceptable,

    /// 407 Proxy Authentication Required
    ProxyAuthenticationRequired,

    /// 408 Request Timeout
    RequestTimeout,

    /// 409 Conflict
    Conflict,

    /// 410 Gone
    Gone,

    /// 411 Length Required
    LengthRequired,

    /// 412 Precondition Failed
    PreconditionFailed,

    /// 413 Payload Too Large
    PayloadTooLarge,

    /// 414 URI Too Long
    UriTooLong,

    /// 415 Unsupported Media Type
    UnsupportedMediaType,

    /// 416 Requested Range Not Satisfiable
    RequestedRangeNotSatisfiable,

    /// 417 Expectation Failed
    ExpectationFailed,

    /// 418 I'm a teapot
    ImATeapot,

    /// 421 Misdirected Request
    MisdirectedRequest,

    /// 422 Unprocessable Entity
    UnprocessableEntity,

    /// 423 Locked
    Locked,

    /// 424 Failed Dependency
    FailedDependency,

    /// 425 Too Early
    TooEarly,

    /// 426 Upgrade Required
    UpgradeRequired,

    /// 428 Precondition Required
    PreconditionRequired,

    /// 429 Too Many Requests
    TooManyRequests,

    /// 431 Request Header Fields Too Large
    RequestHeaderFieldsTooLarge,

    /// 451 Unavailable For Legal Reasons
    UnavailableForLegalReasons,

    /// 500 Internal Server Error
    InternalServerError,

    /// 501 Not Implemented
    NotImplemented,

    /// 502 Bad Gateway
    BadGateway,

    /// 503 Service Unavailable
    ServiceUnavailable,

    /// 504 Gateway Timeout
    GatewayTimeout,

    /// 505 HTTP Version Not Supported
    HttpVersionNotSupported,

    /// 506 Variant Also Negotiates
    VariantAlsoNegotiates,

    /// 507 Insufficient Storage
    InsufficientStorage,

    /// 508 Loop Detected
    LoopDetected,

    /// 510 Not Extended
    NotExtended,

    /// 511 Network Authentication Required
    NetworkAuthenticationRequired,

    /// User defined status.
    Custom {
        /// The status code.
        code: u16,
        /// The HTTP reason phrase.
        reason: &'static str,
    },
}

impl Status {
    /// The HTTP status code.
    pub fn code(self) -> u16 {
        match self {
            Status::Continue => 100,
            Status::SwitchingProtocols => 101,
            Status::EarlyHints => 103,
            Status::Ok => 200,
            Status::Created => 201,
            Status::Accepted => 202,
            Status::NonAuthoritativeInformation => 203,
            Status::NoContent => 204,
            Status::ResetContent => 205,
            Status::PartialContent => 206,
            Status::MultiStatus => 207,
            Status::ImUsed => 226,
            Status::MultipleChoice => 300,
            Status::MovedPermanently => 301,
            Status::Found => 302,
            Status::SeeOther => 303,
            Status::NotModified => 304,
            Status::TemporaryRedirect => 307,
            Status::PermanentRedirect => 308,
            Status::BadRequest => 400,
            Status::Unauthorized => 401,
            Status::PaymentRequired => 402,
            Status::Forbidden => 403,
            Status::NotFound => 404,
            Status::MethodNotAllowed => 405,
            Status::NotAcceptable => 406,
            Status::ProxyAuthenticationRequired => 407,
            Status::RequestTimeout => 408,
            Status::Conflict => 409,
            Status::Gone => 410,
            Status::LengthRequired => 411,
            Status::PreconditionFailed => 412,
            Status::PayloadTooLarge => 413,
            Status::UriTooLong => 414,
            Status::UnsupportedMediaType => 415,
            Status::RequestedRangeNotSatisfiable => 416,
            Status::ExpectationFailed => 417,
            Status::ImATeapot => 418,
            Status::MisdirectedRequest => 421,
            Status::UnprocessableEntity => 422,
            Status::Locked => 423,
            Status::FailedDependency => 424,
            Status::TooEarly => 425,
            Status::UpgradeRequired => 426,
            Status::PreconditionRequired => 428,
            Status::TooManyRequests => 429,
            Status::RequestHeaderFieldsTooLarge => 431,
            Status::UnavailableForLegalReasons => 451,
            Status::InternalServerError => 500,
            Status::NotImplemented => 501,
            Status::BadGateway => 502,
            Status::ServiceUnavailable => 503,
            Status::GatewayTimeout => 504,
            Status::HttpVersionNotSupported => 505,
            Status::VariantAlsoNegotiates => 506,
            Status::InsufficientStorage => 507,
            Status::LoopDetected => 508,
            Status::NotExtended => 510,
            Status::NetworkAuthenticationRequired => 511,
            Status::Custom { code, .. } => code,
        }
    }

    /// Returns the class of a given status.
    pub fn class(self) -> StatusClass {
        match self.code() / 100 {
            1 => StatusClass::Informational,
            2 => StatusClass::Success,
            3 => StatusClass::Redirection,
            4 => StatusClass::ClientError,
            5 => StatusClass::ServerError,
            _ => StatusClass::Custom,
        }
    }

    /// The canonical reason for a given status code.
    pub fn reason(self) -> &'static str {
        match self {
            Status::Continue => "Continue",
            Status::SwitchingProtocols => "Switching Protocols",
            Status::EarlyHints => "Early Hints",
            Status::Ok => "OK",
            Status::Created => "Created",
            Status::Accepted => "Accepted",
            Status::NonAuthoritativeInformation => "Non Authoritative Information",
            Status::NoContent => "No Content",
            Status::ResetContent => "Reset Content",
            Status::PartialContent => "Partial Content",
            Status::MultiStatus => "Multi-Status",
            Status::ImUsed => "Im Used",
            Status::MultipleChoice => "Multiple Choice",
            Status::MovedPermanently => "Moved Permanently",
            Status::Found => "Found",
            Status::SeeOther => "See Other",
            Status::NotModified => "Modified",
            Status::TemporaryRedirect => "Temporary Redirect",
            Status::PermanentRedirect => "Permanent Redirect",
            Status::BadRequest => "Bad Request",
            Status::Unauthorized => "Unauthorized",
            Status::PaymentRequired => "Payment Required",
            Status::Forbidden => "Forbidden",
            Status::NotFound => "Not Found",
            Status::MethodNotAllowed => "Method Not Allowed",
            Status::NotAcceptable => "Not Acceptable",
            Status::ProxyAuthenticationRequired => "Proxy Authentication Required",
            Status::RequestTimeout => "Request Timeout",
            Status::Conflict => "Conflict",
            Status::Gone => "Gone",
            Status::LengthRequired => "Length Required",
            Status::PreconditionFailed => "Precondition Failed",
            Status::PayloadTooLarge => "Payload Too Large",
            Status::UriTooLong => "URI Too Long",
            Status::UnsupportedMediaType => "Unsupported Media Type",
            Status::RequestedRangeNotSatisfiable => "Requested Range Not Satisfiable",
            Status::ExpectationFailed => "Expectation Failed",
            Status::ImATeapot => "I'm a teapot",
            Status::MisdirectedRequest => "Misdirected Request",
            Status::UnprocessableEntity => "Unprocessable Entity",
            Status::Locked => "Locked",
            Status::FailedDependency => "Failed Dependency",
            Status::TooEarly => "Too Early",
            Status::UpgradeRequired => "Upgrade Required",
            Status::PreconditionRequired => "Precondition Required",
            Status::TooManyRequests => "Too Many Requests",
            Status::RequestHeaderFieldsTooLarge => "Request Header Fields Too Large",
            Status::UnavailableForLegalReasons => "Unavailable For Legal Reasons",
            Status::InternalServerError => "Internal Server Error",
            Status::NotImplemented => "Not Implemented",
            Status::BadGateway => "Bad Gateway",
            Status::ServiceUnavailable => "Service Unavailable",
            Status::GatewayTimeout => "Gateway Timeout",
            Status::HttpVersionNotSupported => "HTTP Version Not Supported",
            Status::VariantAlsoNegotiates => "Variant Also Negotiates",
            Status::InsufficientStorage => "Insufficient Storage",
            Status::LoopDetected => "Loop Detected",
            Status::NotExtended => "Not Extended",
            Status::NetworkAuthenticationRequired => "Network Authentication Required",
            Status::Custom { reason, .. } => reason,
        }
    }
}

impl fmt::Display for Status {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} {}", self.code(), self.reason())
    }
}

/// Class of an HTTP status.
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum StatusClass {
    /// A provisional response.
    Informational,

    /// The request has succeeded.
    Success,

    /// Further action needs to be taken to fulfill the request.
    Redirection,

    /// The request cannot be fulfilled due to a client error.
    ClientError,

    /// The server failed to fulfill a valid request.
    ServerError,

    /// A custom status.
    Custom,
}
