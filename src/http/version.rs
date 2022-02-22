use std::fmt;

/// The version of HTTP protocol being used.
#[derive(Copy, Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
#[non_exhaustive]
pub enum Version {
    /// HTTP/0.9
    Http0_9,

    /// HTTP/1.0
    Http1_0,

    /// HTTP/1.1
    Http1_1,

    /// HTTP/2
    Http2,

    /// HTTP/3
    Http3,
}

impl Version {
    /// The string representation of this HTTP version.
    pub fn as_str(self) -> &'static str {
        match self {
            Version::Http0_9 => "HTTP/0.9",
            Version::Http1_0 => "HTTP/1.0",
            Version::Http1_1 => "HTTP/1.1",
            Version::Http2 => "HTTP/2",
            Version::Http3 => "HTTP/3",
        }
    }
}

impl fmt::Display for Version {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}
