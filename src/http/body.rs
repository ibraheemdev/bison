use std::{fmt, io};

use super::{ByteStr, Bytes};

/// The streaming body of an HTTP request or response.
///
/// Data is streamed by iterating over the body, which
/// yields chunks as [`Bytes`](bytes::Bytes).
///
/// ```rust
/// use astra::{Request, Response, Body};
///
/// fn handle(mut req: Request) -> Response {
///     for chunk in req.body_mut() {
///         println!("body chunk {:?}", chunk);
///     }
///
///     Response::new(Body::new("Hello World!"))
/// }
/// ```
pub struct Body(pub(crate) astra::Body);

impl Body {
    /// Create a body from a string literal.
    ///
    /// This method does not allocate.
    ///
    /// ```rust
    /// # use bison::Body;
    /// let body = Body::from_static("Hello world!");
    /// ```
    pub fn from_static(data: &'static str) -> Body {
        Body(astra::Body::new(data))
    }

    /// Create a body from a string or bytes.
    ///
    /// ```rust
    /// # use bison::Body;
    /// let body = Body::new("Hello world!".to_owned());
    /// let body = Body::new(vec![0, 1, 0, 1, 0]);
    /// ```
    pub fn new(data: impl Into<Bytes>) -> Body {
        Body(astra::Body::new(data.into()))
    }

    /// Create an empty body.
    pub fn empty() -> Body {
        Body(astra::Body::empty())
    }

    /// Create a body from an implementor of [`io::Read`].
    ///
    /// ```rust
    /// use bison::{Request, Response, ResponseBuilder, Body};
    /// use std::fs::File;
    ///
    /// fn handle(_request: Request) -> Response {
    ///     let file = File::open("index.html").unwrap();
    ///
    ///     ResponseBuilder::new()
    ///         .header("Content-Type", "text/html")
    ///         .body(Body::wrap_reader(file))
    ///         .unwrap()
    /// }
    /// ```
    pub fn stream<R>(source: R) -> Body
    where
        R: io::Read + Send + 'static,
    {
        Body(astra::Body::wrap_reader(source))
    }
}

impl Iterator for Body {
    type Item = io::Result<Bytes>;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.0.size_hint()
    }
}

impl Default for Body {
    fn default() -> Self {
        Self::empty()
    }
}

impl fmt::Debug for Body {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self.0)
    }
}

impl From<ByteStr> for Body {
    fn from(data: ByteStr) -> Body {
        Body(astra::Body::new(data.into_bytes()))
    }
}

impl From<&str> for Body {
    fn from(data: &str) -> Body {
        Body(astra::Body::new(data.to_owned()))
    }
}

impl From<String> for Body {
    fn from(data: String) -> Body {
        Body(astra::Body::new(data))
    }
}

impl From<&[u8]> for Body {
    fn from(data: &[u8]) -> Body {
        Body(astra::Body::new(data.to_owned()))
    }
}

impl From<Box<[u8]>> for Body {
    fn from(data: Box<[u8]>) -> Body {
        Body(astra::Body::new(data))
    }
}

impl From<Vec<u8>> for Body {
    fn from(data: Vec<u8>) -> Body {
        Body(astra::Body::new(data.to_owned()))
    }
}
